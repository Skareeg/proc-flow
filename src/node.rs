use crate::catalogue::*;
use crate::graph::*;
use axiom::prelude::*;
use dynamic::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use std::sync::*;

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
        output_info: PinInfo,
        context: &Context,
    ) -> Result<Message, String>;
    /// Reacts to an incoming command from another node.
    fn handle_receive(
        &mut self,
        node: &mut Node,
        sender: &PinRef,
        receiver: &PinRef,
        context: &Context,
        message: &Message,
    );
}

///
/// Represents an instance of an individual pin in memory.
///
pub struct Pin {
    /// The general pin information in regards to the graph it comes from.
    pub info: PinInfo,
    /// The id of this specific pin instance.
    pub uuid: uuid::Uuid,
    /// Whether or not this particular output has been designated in the graph to cache its value.
    /// This defaults to true for both inputs and outputs.
    pub cache: Option<bool>,
    /// The links to other pins.
    pub link_pins: std::collections::HashMap<uuid::Uuid, PinRef>,
    /// The values of each of the links.
    pub link_value: std::collections::HashMap<uuid::Uuid, Option<Message>>,
    /// The progress of each of the links.
    pub link_progress: std::collections::HashMap<uuid::Uuid, f32>,
    /// The current value, used for caching.
    pub value: Option<Message>,
    /// The progress until this pin is done computing.
    pub progress: f32,
}

impl Pin {
}

impl Named for Pin {
    fn get_name(&self) -> String {
        self.info.name.clone()
    }
}

///
/// General node container.
///
pub struct Node {
    /// The instance data for this node, pulled from the library.
    pub info: NodeInfo,
    /// The actual input pins to this node.
    pub inputs: std::collections::HashMap<uuid::Uuid, Pin>,
    /// The actual output pins from this node.
    pub outputs: std::collections::HashMap<uuid::Uuid, Pin>,
    /// The actual receive pins to this node.
    pub receives: std::collections::HashMap<uuid::Uuid, Pin>,
    /// The actual send pins from this node.
    pub sends: std::collections::HashMap<uuid::Uuid, Pin>,
    /// The implementation of this particular node instance.
    /// Arc as a threadsafe container.
    /// Mutex to mutate it,
    /// Box to hold it in memory.
    pub process: Arc<Mutex<Box<dyn Nodeable>>>,
    /// A pointer to an mutable catalogue.
    pub catalogue: Arc<Mutex<Catalogue>>,
}

impl Named for Node {
    fn get_name(&self) -> String {
        self.info.graph.name.clone()
    }
}

/// 
/// Each possible command for a given node actor.
/// 
#[derive(Serialize, Deserialize)]
pub enum NodeCommand {
    /// Executes a node, creating the output values.
    /// Will send progress back to caller.
    /// Aid is the commander.
    /// First id is the input pin.
    /// Second id is the output pin.
    /// String is the datatype to request.
    ComputeOutput(Aid, PinRef, PinRef, String),
    // /// Sends an output to another nodes input.
    // /// Aid is the sending node.
    // /// First pin is the input pin.
    // /// Second pin is the output pin for that input.
    // /// String is the datatype sent.
    // /// The message is the value.
    // InputOutputValue(Aid, PinRef, PinRef, String, Message),
    /// Sends an output to another nodes input.
    /// Aid is the sending node.
    /// Pin is the input pin.
    /// String is the datatype sent.
    /// The message is the value.
    InputValue(Aid, PinRef, String, Message),
    /// Sends a message of some sort to a receiver.
    /// Aid is the sending node.
    /// First id is the sending pin.
    /// Second id is the receiving pin.
    /// The dynamic is the message.
    ReceiverMessage(Aid, PinRef, PinRef, Message),
    /// Requests the progress of a node.
    /// These are primarily sent by external actors, or the graph editor actor.
    /// Aid is the requestor.
    /// Pin is the output whose progress is updating.
    RequestProgress(Aid, PinRef),
    /// Provides the progress of a given node.
    /// These are sent automatically during computes.
    /// Aid is the progressing node.
    /// Pin is the output whose progress is updating.
    /// Float is the progress.
    UpdateProgress(Aid, PinRef, f32),
}

use log::*;

fn send_input_output(context: &Context, commander: Aid, output: PinRef, input: PinRef, datatype: String, msg: Message) {
    match commander.send_new(NodeCommand::InputValue(context.aid.clone(), input.clone(), datatype.clone(), msg)) {
        Ok(()) => trace!("sent inputoutput from node actor {:?} to node actor {:?}, pin {:?} to pin {:?} with datatype {}", context.aid.clone(), commander.clone(), output.clone(), input.clone(), datatype.clone()),
        Err(e) => error!("unable to send inputoutput from node actor {:?} to node actor {:?}, pin {:?} to pin {:?} with datatype {}", context.aid.clone(), commander.clone(), output.clone(), input.clone(), datatype.clone())
    };
}


fn pin_vec_to_hashmap(pins: Vec<Pin>) -> std::collections::HashMap<uuid::Uuid, Pin> {
    let mut map = std::collections::HashMap::new();
    for p in pins {
        map.insert(p.uuid, p);
    }
    map
}

///
/// Implementation for a node.
///
impl Node {
    pub fn new(info: NodeInfo, process: Box<dyn Nodeable>, catalogue: Arc<Mutex<Catalogue>>) -> Self {
        let cat = catalogue.lock().unwrap();
        let (vinputs, voutputs) = process.get_io(&cat);
        let (vreceives, vsends) = process.get_rs(&cat);
        let inputs = pin_vec_to_hashmap(vinputs);
        let outputs = pin_vec_to_hashmap(voutputs);
        let receives = pin_vec_to_hashmap(vreceives);
        let sends = pin_vec_to_hashmap(vsends);
        Self {
            info,
            inputs,
            outputs,
            receives,
            sends,
            process: Arc::new(Mutex::new(process)),
            catalogue: catalogue.clone()
        }
    }
    ///
    /// Handle messages sent by other actors.
    ///
    pub async fn handle(mut self, context: Context, message: Message) -> ActorResult<Self> {
        if let Some(msg) = message.content_as::<NodeCommand>() {
            match &*msg {
                // This is a received request to process our outputs if needed and send them forward.
                NodeCommand::ComputeOutput(commander, input, output, datatype) => {
                    // If this is a request for the ouput of a pin.
                    match output.pin {
                        Some(requested_output_pin) => {
                            // Gather the needed information prematurely, or the borrow checker will have a field day.
                            let output_info: PinInfo;
                            let output_uuid: uuid::Uuid;
                            let output_value: Option<Message>;
                            // Reference to self, and the pin as well, must go out of scope.
                            {
                                let output_pin = self.outputs.get_mut(&requested_output_pin);
                                match output_pin {
                                    // The pin exists. Get the requried information.
                                    Some(output_pin) => {
                                        output_info = output_pin.info.clone();
                                        output_uuid = output_pin.uuid;
                                        output_value = output_pin.value.clone();
                                    },
                                    // The pin does not exist. Return after logging an error.
                                    None => {
                                        error!(
                                            "node actor {} does not have outpin pin with uuid of {}",
                                            &context.aid, requested_output_pin
                                        );
                                        return Ok(Status::done(self));
                                    }
                                }
                            }
                            // Is the datatype correct?
                            if &*output_info.datatype == &*datatype {
                                // Is there already a value?
                                // TODO: Make this a setting on node pins.
                                match output_value {
                                    // Send the value already there, effectively caching it.
                                    Some(output_value) => {
                                        send_input_output(&context, commander.clone(), output.clone(), input.clone(), output_info.datatype.clone(), output_value.clone());
                                    }
                                    None => {
                                        let process = self.process.clone();
                                        let new_output_value = process.lock().unwrap().compute_output(
                                            &self,
                                            output_info.clone(),
                                            &context
                                        );
                                        match new_output_value {
                                            Ok(new_output_value) => {
                                                let output_pin = self.outputs.get_mut(&requested_output_pin).unwrap();
                                                output_pin.value = Some(new_output_value.clone());
                                                send_input_output(&context, commander.clone(), output.clone(), input.clone(), output_info.datatype.clone(), new_output_value.clone());
                                            },
                                            Err(e) => error!("could not calculate output value for node actor {:?} pin {} because of reason: {}", &context.aid, requested_output_pin, e)
                                        };
                                    }
                                }
                            } else {
                                error!("incorrect requested datatype from node actor {:?} pin {} to node actor {:?} pin {}", &context.aid, output_uuid, &commander, input.pin.unwrap());
                            }
                        }
                        ,None => {
                        }
                    }
                    // TODO: Add the reply for request of node value.
                }
                // This is a reply from a compute output request, setting the value of the input.
                NodeCommand::InputValue(commander, input, datatype, message) => {
                    let ipin: Option<&mut Pin> = self
                        .inputs.get_mut(&input.pin.unwrap());
                    match ipin {
                        Some(ipin) => {
                            if &*ipin.info.datatype == &*datatype {
                                ipin.value = Some(message.clone());
                            } else {
                                error!("incorrect datatype sent from actor {:?} to actor {:?}: pin {}", commander, &context.aid, ipin.uuid);
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
                    let process = self.process.clone();
                    process.lock().unwrap().handle_receive(
                        &mut self,
                        &sender,
                        &receiver,
                        &context,
                        message,
                    );
                }
                NodeCommand::RequestProgress(requestor, output) => {
                    let progress = self.outputs.get(&output.pin.unwrap()).unwrap().progress.clone();
                    match requestor.send_new(NodeCommand::UpdateProgress(context.aid.clone(), output.clone(), progress.clone())) {
                        Ok(()) => trace!("update progress ({}) sent from {:?} to {:?}", progress, &context.aid, requestor),
                        Err(e) => error!("could not send update progress ({}) from {:?} to {:?}: {:?}", progress.clone(), &context.aid, requestor, e)
                    };
                }
                NodeCommand::UpdateProgress(progressor, output, progress) => {
                    self.inputs
                        .values_mut()
                        .for_each(|input: &mut Pin| {
                            if let Some(output) = input.link_progress.get_mut(&output.pin.unwrap()) {
                                *output = *progress;
                                let total_progress: f32 = input.link_progress.values().sum();
                                let link_count = input.link_progress.len() as f32;
                                input.progress = total_progress / link_count;
                            }
                        });
                }
            };
        }
        Ok(Status::done(self))
    }
}