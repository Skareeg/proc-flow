// use crate::node::*;

use crate::catalogue::*;
// use crate::graph::*;

use axiom::prelude::*;

use crossbeam::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::*;

///
/// Proc Flow main engine gateway.
/// This class acts more as a communicator between the external world and the Proc Flow engine.
/// The true engine behind Proc Flow is actually the axiom ActorSystem contained inside.
/// This class allows outside environments to properly communicate requests and intent to the engine via a controller actor.
///
pub struct Engine {
    /// Actor System in which to run nodes.
    pub system: ActorSystem,
    /// Mutable shared reference to the Catalogue.
    pub catalogue: Arc<Mutex<Catalogue>>,
    /// Sending channel to send things.
    pub send_to_controller: Sender<Message>,
    /// Recieving channel to get things back.
    pub recv_from_controller: Receiver<Message>,
    /// Controller node handle.
    pub controller: Aid,
    /// Controls whether or not this engine should keep waiting for new messages.
    pub keep_waiting: Arc<Mutex<bool>>,
}

impl Engine {
    ///
    /// Constructs and initiates a Proc Flow engine and associated controller.
    ///
    pub fn new() -> Self {
        // Create engine state.
        let system = ActorSystem::create(ActorSystemConfig::default());
        let catalogue = Arc::new(Mutex::new(Catalogue::new()));
        let (send_to_controller, recv_from_engine) = crossbeam::unbounded();
        let (send_to_engine, recv_from_controller) = crossbeam::unbounded();
        let nodes = HashMap::new();

        catalogue.lock().unwrap().load_default_libraries();

        let keep_waiting = Arc::new(Mutex::new(true));

        // Create and spawn the controller.
        let controller_state = Controller {
            nodes,
            send_to_engine,
            recv_from_engine,
            catalogue: catalogue.clone(),
            keep_waiting: keep_waiting.clone(),
        };
        let controller = system
            .spawn()
            .name("controller")
            .with(controller_state, Controller::handle)
            .expect("could not create engine controller");

        // Return newly contructed engine.
        Self {
            system,
            catalogue,
            send_to_controller,
            recv_from_controller,
            controller,
            keep_waiting,
        }
    }
    pub fn test_5(&mut self) {
        let _ = self.controller.send_new_after(
            ControllerCommand::GiveMe5,
            std::time::Duration::from_secs(5),
        );
        info!("Prepared to send a giveme5 request");
        if let Some(num) = self.recv_from_controller.recv().unwrap().content_as::<u64>() {
            info!("Received the number {}", num);
        }
    }
    pub fn boot_graph(
        &mut self,
        id: uuid::Uuid,
        version: u64,
        instance_id: uuid::Uuid,
    ) -> Option<Aid> {
        let _ =
            self.controller
                .send_new(ControllerCommand::BootGraph(id, version, instance_id, None));
        if let Some(msg) = self.recv_from_controller.recv().unwrap().content_as::<ControllerResponse>() {
            match &*msg {
                ControllerResponse::GraphBooted(_instance, actor) => {
                    return actor.clone();
                }
                _ => {
                    error!("bad response on boot graph request");
                }
            }
        }
        None
    }
    pub fn boot_cluster(&mut self, _port: u64) {
        unimplemented!();
    }
    pub fn set_input_pin_value(&mut self, node_actor: Aid, input: uuid::Uuid, value: Option<Message>, datatype: String) {
        info!("engine set input pin value");
        match self.controller.send_new(ControllerCommand::SetInputPinValue(node_actor.clone(), input, value, datatype)) {
            Ok(()) => {
                if let Some(msg) = self.recv_from_controller.recv().unwrap().content_as::<ControllerResponse>() {
                    match &*msg {
                        ControllerResponse::InputPinSet => {
                            return;
                        }
                        _ => {
                            error!("bad response on get output pin value request to controller");
                        }
                    }
                };
            }
            Err(e) => {
                error!("could not send message to set input pin {:?} value on node actor {:?}: {}", input.clone(), node_actor.clone(), e.to_string());
            }
        }
    }
    pub fn compute_output_pin_value(&mut self, node_actor: Aid, output: uuid::Uuid, parameters: Option<Message>) -> Option<Message> {
        info!("engine get output pin value");
        match self.controller.send_new(ControllerCommand::ComputeOutputPinValue(node_actor.clone(), output, parameters)) {
            Ok(()) => {
                if let Some(msg) = self.recv_from_controller.recv().unwrap().content_as::<ControllerResponse>() {
                    match &*msg {
                        ControllerResponse::OutputValue(_node_actor, value) => {
                            return value.clone();
                        }
                        _ => {
                            error!("bad response on get output pin value request to controller");
                        }
                    }
                };
            }
            Err(e) => {
                error!("could not send message to get output pin {:?} value on node actor {:?}: {}", output.clone(), node_actor.clone(), e.to_string());
            }
        }
        None
    }
    pub fn send_value(&mut self, node_actor: Aid, receiver: uuid::Uuid, value: Option<Message>) {
        info!("engine send value");
        match self.controller.send_new(ControllerCommand::SendValue(node_actor.clone(), receiver.clone(), value)) {
            Ok(()) => {
                if let Some(msg) = self.recv_from_controller.recv().unwrap().content_as::<ControllerResponse>() {
                    match &*msg {
                        ControllerResponse::ValueSent => {
                            info!("engine value sent")
                        }
                        _ => {
                            error!("bad response on send value request to controller");
                        }
                    }
                };
            }
            Err(e) => {
                error!("could not send message to set receiver pin {:?} on node actor {:?}: {}", receiver.clone(), node_actor.clone(), e.to_string());
            }
        }
    }
    /// Tells the engine that it can stop waiting for new messages.
    pub fn stop_waiting(&mut self) {
        *(self.keep_waiting.lock().unwrap()) = false;
    }
    /// Will wait until the nodes are done processing.
    /// If a timeout is specified, the function will return the ShutdownResult.
    /// If a timeout is not specificed, the function will keep polling the function until the engine is told to stop waiting, when it will then return the last ShutdownResult.
    pub fn wait(&mut self, timeout: impl Into<Option<std::time::Duration>>) -> ShutdownResult {
        let timeout = timeout.into();
        match timeout {
            None => {
                let mut res = self.system.trigger_and_await_shutdown(None);
                while *(self.keep_waiting.lock().unwrap()) && res != ShutdownResult::Panicked {
                    res = self.system.trigger_and_await_shutdown(None);
                }
                res
            }
            _ => {
                self.system.trigger_and_await_shutdown(timeout)
            }
        }
    }
}

///
/// Each possible command from the engine or a node to the controller.
///
#[derive(Serialize, Deserialize)]
pub enum ControllerCommand {
    /// Test command to ensure that the controller is functional.
    /// TODO: Remove when the system is working.
    GiveMe5,
    /// Boots up a graph instance to gather IO from.
    /// UUID is the graph's UUID in the catalogue.
    /// Number is the version number to load.
    /// UUID is the instance of that graph.
    /// Id is the node that requested this boot, if available.
    BootGraph(uuid::Uuid, u64, uuid::Uuid, Option<Aid>),
    /// Sends a message to its target, including remote destinations.
    /// TODO: Actual cluster implementation.
    RouteMessage(Aid, Aid, Message),
    /// Computes or gets and existing output pin's value.
    /// First id is the node actor to grab from.
    /// Second is the UUID of the pin to grab from.
    /// Message is the arguments to the output pin.
    ComputeOutputPinValue(Aid, uuid::Uuid, Option<Message>),
    /// Sets the value of a nodes input.
    /// First id is the node actor to set.
    /// Second is the UUID of the pin to set.
    /// Message is the arguments to the output pin.
    /// String is the datatype of the message.
    SetInputPinValue(Aid, uuid::Uuid, Option<Message>, String),
    /// Sends a value to a nodes receiver pins. 
    /// First id is the node actor to send to.
    /// Second is the UUID of the pin to send to.
    /// Message is the message to send.
    SendValue(Aid, uuid::Uuid, Option<Message>),
    /// Initiates a reqeust reply poll.
    /// Id is the id of the request itself.
    /// TODO: Is this needed?
    REQREP(uuid::Uuid),
}

///
/// Each possible response from the controller.
///
#[derive(Serialize, Deserialize)]
pub enum ControllerResponse {
    /// Presents that a graph was booted correctly.
    GraphBooted(uuid::Uuid, Option<Aid>),
    /// Presents a value from the pin of an output.
    OutputValue(Aid, Option<Message>),
    /// Presents that a pin's value was set sucessfully.
    InputPinSet,
    /// TODO Proper comment here.
    ValueSent,
    /// Tells the engine that nodes are fine with being shutdown, as told by one of the nodes themselves.
    CanShutdown,
}

///
/// Actor that acts as the actual control system and internal engine gateway to the ProcFlow system.
///
pub struct Controller {
    /// Map of root graph nodes that are active in the system.
    pub nodes: HashMap<uuid::Uuid, Aid>,
    /// TX to the Proc Flow engine structure.
    pub send_to_engine: Sender<Message>,
    /// TX to the Proc Flow engine structure.
    pub recv_from_engine: Receiver<Message>,
    /// Reference to the node library.
    pub catalogue: Arc<Mutex<Catalogue>>,
    /// Controls whether or not the engine is still running.
    pub keep_waiting: Arc<Mutex<bool>>,
}

use log::*;
use crate::node::NodeResponse;

impl Controller {
    ///
    /// Handle messages sent by other actors.
    ///
    pub async fn handle(self, context: Context, message: Message) -> ActorResult<Self> {
        if let Some(msg) = message.content_as::<ControllerCommand>() {
            match &*msg {
                ControllerCommand::GiveMe5 => {
                    let _ = self.send_to_engine.send(Message::new(5 as u64));
                }
                ControllerCommand::BootGraph(graph_id, version, instance_id, requestor) => {
                    info!(
                        "boot sequence initiated for graph {} version {}",
                        graph_id, version
                    );

                    // Attempt to find the graph in the catalogue.
                    let graph_ref: Option<crate::graph::GraphRef>;
                    {
                        let cat = self.catalogue.lock().unwrap();
                        graph_ref = cat.get_graph_ref(graph_id.clone(), version.clone());
                    }

                    match graph_ref {
                        Some(graph_ref) => {
                            info!(
                                "graph {} found in catalogue with name {}",
                                graph_id,
                                graph_ref.name.clone()
                            );

                            // Check if the library is internal or not.
                            let internal_lib =
                                uuid::Uuid::parse_str("b0fa443c-20d0-4c2a-acf9-76c63af3cbed")
                                    .unwrap();
                            if graph_ref.library.expect("library does not have an UUID")
                                == internal_lib
                            {
                                // Create an internal library node.
                                let node = crate::nodes::create(
                                    context.aid,
                                    self.catalogue.clone(),
                                    graph_id.clone(),
                                    version.clone(),
                                    instance_id.clone(),
                                );
                                match node {
                                    Some(node) => {
                                        match context
                                            .system
                                            .spawn()
                                            .name(node.info.graph.name.clone())
                                            .with(node, crate::node::Node::handle)
                                        {
                                            Ok(actor) => {
                                                info!("internal graph {} : {} version {} node actor spawned", graph_id, graph_ref.name.clone(), version.clone());
                                                match requestor {
                                                    Some(requestor) => {
                                                        let _ = requestor.send_new(
                                                            ControllerResponse::GraphBooted(
                                                                instance_id.clone(),
                                                                Some(actor),
                                                            ),
                                                        );
                                                    }
                                                    None => {
                                                        let _ = self.send_to_engine.send(Message::new(
                                                            ControllerResponse::GraphBooted(
                                                                instance_id.clone(),
                                                                Some(actor),
                                                            ),
                                                        ));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!("internal graph {} : {} version {} node actor could not be spawned: {:?}", graph_id, graph_ref.name.clone(), version.clone(), e);
                                                match requestor {
                                                    Some(requestor) => {
                                                        let _ = requestor.send_new(
                                                            ControllerResponse::GraphBooted(
                                                                instance_id.clone(),
                                                                None,
                                                            ),
                                                        );
                                                    }
                                                    None => {
                                                        let _ = self.send_to_engine.send(Message::new(
                                                            ControllerResponse::GraphBooted(
                                                                instance_id.clone(),
                                                                None,
                                                            ),
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    None => {
                                        error!("internal graph {} : {} version {} could not be created", graph_id, graph_ref.name.clone(), version.clone());
                                        match requestor {
                                            Some(requestor) => {
                                                let _ = requestor.send_new(
                                                    ControllerResponse::GraphBooted(
                                                        instance_id.clone(),
                                                        None,
                                                    ),
                                                );
                                            }
                                            None => {
                                                let _ = self.send_to_engine.send(Message::new(
                                                    ControllerResponse::GraphBooted(
                                                        instance_id.clone(),
                                                        None,
                                                    ),
                                                ));
                                            }
                                        }
                                    }
                                }
                            } else {
                                // Create a user created graph node, a single node that represents the whole graph.

                                // Make sure the specific version of the graph exists.
                                let has_version: bool;
                                {
                                    let cat = self.catalogue.lock().unwrap();
                                    has_version = cat.has_graph_version(&graph_ref);
                                }
                                if has_version {
                                    info!(
                                        "graph {} : {} version {} found in catalogue",
                                        graph_id,
                                        graph_ref.name.clone(),
                                        version.clone()
                                    );
                                    // TODO Implement dynamic user graph booting, likely as a graph node.
                                } else {
                                    error!(
                                        "graph {} : {} does not have version {} in catalogue",
                                        graph_id,
                                        graph_ref.name.clone(),
                                        version.clone()
                                    );
                                    match requestor {
                                        Some(requestor) => {
                                            let _ = requestor.send_new(
                                                ControllerResponse::GraphBooted(
                                                    instance_id.clone(),
                                                    None,
                                                ),
                                            );
                                        }
                                        None => {
                                            let _ = self.send_to_engine.send(Message::new(
                                                ControllerResponse::GraphBooted(
                                                    instance_id.clone(),
                                                    None,
                                                ),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        None => {
                            error!("graph {} does not exist in the catalogue", graph_id);
                            match requestor {
                                Some(requestor) => {
                                    let _ = requestor.send_new(ControllerResponse::GraphBooted(
                                        instance_id.clone(),
                                        None,
                                    ));
                                }
                                None => {
                                    let _ = self.send_to_engine.send(Message::new(
                                        ControllerResponse::GraphBooted(instance_id.clone(), None),
                                    ));
                                }
                            }
                        }
                    }
                }
                ControllerCommand::RouteMessage(sender, receiver, message) => {
                    trace!("message from {:?} to {:?}", sender, receiver);
                    match receiver.send(message.clone()) {
                        Ok(()) => {}
                        Err(_e) => {}
                    }
                }
                ControllerCommand::SetInputPinValue(node_actor, pin_id, parameters, datatype) => {
                    info!("controller set input pin value");
                    match node_actor.send_new(crate::node::NodeCommand::InputValue(context.aid, pin_id.clone(), datatype.clone(), parameters.clone())) {
                        Err(e) => error!("controller could not send command to node actor {} to set input of pin {}: {}", node_actor.clone(), pin_id.clone(), e.to_string()),
                        _ => {}
                    }
                }
                ControllerCommand::ComputeOutputPinValue(node_actor, pin_id, parameters) => {
                    // TODO! Determine getting an output should block! Make both versions? Poll returns current output. Compute computes it!
                    info!("controller get output pin value");
                    match node_actor.send_new(crate::node::NodeCommand::ComputeOutput(context.aid, pin_id.clone(), parameters.clone())) {
                        Err(e) => error!("controller could not send command to node actor {} to get output of pin {}: {}", node_actor.clone(), pin_id.clone(), e.to_string()),
                        _ => {}
                    }
                }
                ControllerCommand::SendValue(node_actor, pin_id, parameters) => {
                    info!("controller send value");
                    match node_actor.send_new(crate::node::NodeCommand::ReceiverMessage(context.aid, pin_id.clone(), parameters.clone())) {
                        Err(e) => error!("controller could not send command to node actor {} to send value to receiver pin {}: {}", node_actor.clone(), pin_id.clone(), e.to_string()),
                        _ => {}
                    }
                }
                // TODO: Remove?
                ControllerCommand::REQREP(_id) => {
                }
            }
        }
        if let Some(msg) = message.content_as::<NodeResponse>() {
            match &*msg {
                NodeResponse::OutputPinValue(node_actor, output_pin, value) => {
                    match self.send_to_engine.send(Message::new(ControllerResponse::OutputValue(node_actor.clone(), value.clone()))) {
                        Err(e) => error!("controller could not send node actor {} pin {} value to engine channel: {}", node_actor.clone(), output_pin.clone(), e.to_string()),
                        _ => {}
                    }
                }
                NodeResponse::InputPinSet => {
                    match self.send_to_engine.send(Message::new(ControllerResponse::InputPinSet)) {
                        Err(e) => error!("controller could not get input pin value set confirmation to engine channel: {}", e.to_string()),
                        _ => {}
                    }
                }
                NodeResponse::Received => {
                    match self.send_to_engine.send(Message::new(ControllerResponse::ValueSent)) {
                        Err(e) => error!("controller could not get send value confirmation to engine channel: {}", e.to_string()),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(Status::done(self))
    }
}
