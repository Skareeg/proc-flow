use crate::catalogue::*;
use crate::graph::*;
use axiom::prelude::*;
use dynamic::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use std::sync::*;

use std::collections::HashMap;

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
        &mut self,
        node: &mut Node,
        output_info: PinInfo,
        context: &Context,
        parameter: &Option<Message>,
    ) -> Result<Option<Message>, String>;
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
    pub cache: bool,
    /// The links to other node actors for each link to a pin.
    pub link_nodes: std::collections::HashMap<uuid::Uuid, Aid>,
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
    pub fn new_io_basic(info: PinInfo) -> Self {
        Self {
            info,
            uuid: uuid::Uuid::new_v4(),
            cache: true,
            link_nodes: HashMap::new(),
            link_pins: HashMap::new(),
            link_value: HashMap::new(),
            link_progress: HashMap::new(),
            value: None,
            progress: 0.0
        }
    }
    pub fn new_rs_basic(info: PinInfo) -> Self {
        Self {
            info,
            uuid: uuid::Uuid::new_v4(),
            cache: false,
            link_nodes: HashMap::new(),
            link_pins: HashMap::new(),
            link_value: HashMap::new(),
            link_progress: HashMap::new(),
            value: None,
            progress: 0.0
        }
    }
}

impl Named for Pin {
    fn get_name(&self) -> String {
        self.info.name.clone()
    }
}

/// Information for a live instance of a node in the actor system.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeInstanceInfo {
    /// The id of this individual instance of the node.
    pub uuid: uuid::Uuid,
    /// Data variables used by the node to store data that is not held on any inputs, including large array data, matrix data, and a string or many strings pointing to relative file paths or not-recommended absolute file paths of data files.
    /// Basically just whatever the node needs to hold.
    pub data: Option<Vec<Datum>>,
    /// The reference to the graph that this node instance represents.
    pub graph: GraphRef,
}

impl NodeInstanceInfo {
    pub fn from_info(info: &NodeInfo) -> Self {
        Self {
            uuid: info.uuid.clone(),
            data: info.data.clone(),
            graph: info.graph.clone()
        }
    }
}

///
/// General node container.
///
pub struct Node {
    /// The instance data for this node, pulled from the library.
    pub info: NodeInstanceInfo,
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
    pub process: Arc<Mutex<Box<dyn Nodeable + Send + Sync + 'static>>>,
    /// A pointer to an mutable catalogue.
    pub catalogue: Arc<Mutex<Catalogue>>,
    /// Controller that this node belongs to.
    pub controller: Aid,
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
    /// Optional message is a parameter or list of parameters to send to the output pin's function.
    ComputeOutput(Aid, PinRef, PinRef, String, Option<Message>),
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
    pub fn new(info: NodeInstanceInfo, process: Box<dyn Nodeable + Send + Sync>, catalogue: Arc<Mutex<Catalogue>>, controller: Aid) -> Self {
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
            catalogue: catalogue.clone(),
            controller
        }
    }
    ///
    /// Handle messages sent by other actors.
    ///
    pub async fn handle(mut self, context: Context, message: Message) -> ActorResult<Self> {
        if let Some(msg) = message.content_as::<NodeCommand>() {
            match &*msg {
                // This is a received request to process our outputs if needed and send them forward.
                NodeCommand::ComputeOutput(commander, input, output, datatype, parameter) => {
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
                                            &mut self,
                                            output_info.clone(),
                                            &context,
                                            &parameter
                                        );
                                        match new_output_value {
                                            Ok(new_output_value) => {
                                                let output_pin = self.outputs.get_mut(&requested_output_pin).unwrap();
                                                output_pin.value = new_output_value.clone();
                                                send_input_output(&context, commander.clone(), output.clone(), input.clone(), output_info.datatype.clone(), Message::new(new_output_value.clone()));
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
                                ipin.value = (*message.content_as::<Option<Message>>().expect("input to node was not an optional message arc")).clone();
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