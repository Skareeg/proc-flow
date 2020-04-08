use crate::catalogue::*;
use crate::graph::*;
use axiom::prelude::*;
use dynamic::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

extern crate crossbeam;

pub trait Named {
    fn get_name(&self) -> String;
}

///
/// Implements a struct that acts as a node process.
///
pub trait Nodeable {
    /// Gets the default input/output pins for this node.
    fn get_io(&self, catalogue: &Catalogue) -> (std::vec::Vec<Pin>, std::vec::Vec<Pin>);
    /// Gets the default receive/send pins for this node.
    fn get_rs(&self, catalogue: &Catalogue) -> (std::vec::Vec<Pin>, std::vec::Vec<Pin>);
    /// Computes one of the outputs for a pin.
    /// This may have different behavior for each node, as some may calculate all of their outputs at once, and others may only calculate what they need.
    fn compute_output(
        &self,
        node: &Node,
        catalogue: std::sync::Arc<Catalogue>,
        output: PinInfo,
    ) -> Result<Message, String>;
    /// Reacts to an incoming command from another node.
    fn handle_receive(
        &self,
        node: &mut Node,
        catalogue: std::sync::Arc<Catalogue>,
        sender: &PinRef,
        receiver: &PinRef,
        context: Context,
        message: &Message,
    );
}

///
/// Represents an instance of an individual pin in memory.
///
pub struct Pin {
    /// The general pin information in regards to the graph it comes from.
    pub info: PinInfo,
    /// The information about this pin instance.
    pub instance: PinRef,
    pub uuid: uuid::Uuid,
    pub links: Vec<PinRef>,
    pub value: Option<Message>,
}

impl Named for Pin {
    fn get_name(&self) -> String {
        match self.index {
            0 => self.info.name.clone(),
            _ => format!("{}[{}]", self.info.name.clone(), self.index),
        }
    }
}

///
/// General node container.
///
pub struct Node {
    /// The instance data for this node, pulled from the library.
    pub info: NodeInfo,
    /// The actual input and receive pins to this node.
    pub inputs: std::collections::HashMap<uuid::Uuid, Pin>,
    /// The actual output and send pins from this node.
    pub outputs: std::collections::HashMap<uuid::Uuid, Pin>,
    /// The implementation of this particular node instance.
    pub process: Box<dyn Nodeable>,
    /// How far along this node is to computing it's last request.
    /// TODO Move this into the pins and handle requests for progress by pin.
    pub progress: f32,
    /// A pointer to the local catalogue.
    /// In a normal actor system this would be its own actor and we would just request it is needed, however that would have far too much latency.
    /// With that in mind, it is going into a local shared thread memory location.
    /// TODO Instead of a catalogue, just have the functions to load node information from a file and lazy load as needed.
    pub catalogue: std::sync::Arc<Catalogue>,
}

impl Named for Node {
    fn get_name(&self) -> String {
        self.info.graph.name.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub enum NodeCommand {
    /// Executes a node, creating the output values.
    /// Will send progress back to caller.
    /// Aid is the commander.
    /// First id is the input pin.
    /// Second id is the output pin.
    /// String is the datatype to request.
    ComputeOutput(Aid, PinRef, PinRef, String),
    /// Sends an output to another nodes input.
    /// Aid is the sending node.
    /// First id is the output pin.
    /// Second id is the input pin.
    /// String is the datatype sent.
    /// The message is the value.
    InputOutput(Aid, PinRef, PinRef, String, Message),
    /// Sends a message of some sort to a receiver.
    /// Aid is the sending node.
    /// First id is the sending pin.
    /// Second id is the receiving pin.
    /// The dynamic is the message.
    ReceiverMessage(Aid, PinRef, PinRef, Message),
    /// Requests the progress of a node.
    /// Aid is the requestor.
    RequestProgress(Aid),
    /// Provides the progress of a given node.
    /// Aid is the progressing node, and f32 is the progress.
    UpdateProgress(Aid, f32),
}

use log::*;

///
/// Implementation for a node.
///
impl Node {
    ///
    /// Handle messages sent by other actors.
    ///
    pub async fn handle(mut self, context: Context, message: Message) -> ActorResult<Self> {
        if let Some(msg) = message.content_as::<NodeCommand>() {
            match &*msg {
                // This is a received request to process our outputs if needed and send them forward.
                NodeCommand::ComputeOutput(commander, input, output, datatype) => {
                    // If this is a request for the ouput of a pin.
                    if let Some(requested_output_pin) = output.pin {
                        let output_pin = self.outputs.get_mut(&requested_output_pin);
                        // Does this pin exist?
                        match output_pin {
                            // Send or calculate and send.
                            Some(output_pin) => {
                                let output_value = output_pin.value;
                                // Is there already a value?
                                match output_value {
                                    // Send the value already there, effectively caching it.
                                    Some(output_value) => {
                                        // Send
                                    }
                                    None => {
                                        let new_output_value = self.process.compute_output(
                                            &self,
                                            self.catalogue.clone(),
                                            output_pin.info.clone(),
                                        );
                                        match new_output_value {
                                            Ok(new_output_value) => {
                                                output_pin.value = Some(new_output_value);
                                                // Send
                                            },
                                            Err(e) => error!("could not calculate output value for node actor {:?} pin {} because of reason: {}", &context.aid, requested_output_pin, e)
                                        };
                                    }
                                }
                            }
                            // Pin does not exist.
                            None => {
                                error!(
                                    "node actor {} does not have outpin pin with uuid of {}",
                                    &context.aid, requested_output_pin
                                );
                            }
                        }
                    }
                }
                NodeCommand::InputOutput(commander, output, input, datatype, message) => {
                    let ipin: Option<&mut Pin> = self
                        .inputs
                        .iter_mut()
                        .find(|inp| inp.uuid == input.pin.unwrap());
                    match ipin {
                        Some(ipin) => {
                            if ipin.info.datatype.cmp(datatype) == std::cmp::Ordering::Equal {
                                ipin.value = Some(message.clone());
                            } else {
                                if let Some(opin) = output.pin {
                                    if let Some(oindex) = output.index {
                                        error!("incorrect datatype sent from actor {:?}: pin {}: index {} to actor {:?}: pin {}", commander, opin, oindex, &context.aid, ipin.uuid);
                                    } else {
                                        error!("incorrect datatype sent from actor {:?}: pin {} to actor {:?}: pin {}", commander, opin, &context.aid, ipin.uuid);
                                    }
                                }
                                if let Some(ovalue) = output.value {
                                    error!("incorrect datatype sent from actor {:?}: value {} to actor {:?}: pin {}", commander, ovalue, &context.aid, ipin.uuid);
                                }
                            }
                        }
                        None => error!(
                            "node actor {:?} does not have input pin with uuid of {}",
                            &context.aid,
                            input.pin.unwrap()
                        ),
                    };
                }
                NodeCommand::ReceiverMessage(commander, sender, receiver, message) => {
                    self.process.handle_receive(
                        &mut self,
                        self.catalogue.clone(),
                        &sender,
                        &receiver,
                        context,
                        message,
                    );
                }
                NodeCommand::RequestProgress(requestor) => {}
                NodeCommand::UpdateProgress(progressor, progress) => {
                    self.progress = *progress;
                }
            }
        }
        Ok(Status::done(self))
    }
}
