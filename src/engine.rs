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
    /// Sending channel to request for things.
    pub send: Sender<Message>,
    /// Recieving channel to get things back.
    pub recv: Receiver<Message>,
    /// Controller node handle.
    pub controller: Aid,
}

use crate::graph::PinRef;

impl Engine {
    ///
    /// Constructs and initiates a Proc Flow engine and associated controller.
    ///
    pub fn new() -> Self {
        // Create engine state.
        let system = ActorSystem::create(ActorSystemConfig::default());
        let catalogue = Arc::new(Mutex::new(Catalogue::new()));
        let (send, recv) = crossbeam::unbounded();
        let nodes = HashMap::new();

        catalogue.lock().unwrap().load_default_libraries();

        // Create and spawn the controller.
        let controller_state = Controller {
            nodes,
            sender: send.clone(),
            receiver: recv.clone(),
            catalogue: catalogue.clone(),
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
            send,
            recv,
            controller,
        }
    }
    pub fn test_5(&mut self) {
        let _ = self.controller.send_new_after(
            ControllerCommand::GiveMe5,
            std::time::Duration::from_secs(5),
        );
        info!("Prepared to send a giveme5 request");
        if let Some(num) = self.recv.recv().unwrap().content_as::<u64>() {
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
        if let Some(msg) = self.recv.recv().unwrap().content_as::<ControllerResponse>() {
            match &*msg {
                ControllerResponse::GraphBooted(_instance, actor) => {
                    return actor.clone();
                }
            }
        }
        None
    }
    pub fn boot_cluster(&mut self, _port: u64) {
        unimplemented!();
    }
    pub fn get_output_pin_value(&mut self, node_actor: Aid, output: uuid::Uuid) -> Option<Message> {
        unimplemented!();
    }
    pub fn wait(&mut self) {
        self.system.await_shutdown(None);
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
}

///
/// Each possible response from the controller.
///
#[derive(Serialize, Deserialize)]
pub enum ControllerResponse {
    /// Presents that a graph was booted correctly.
    GraphBooted(uuid::Uuid, Option<Aid>),
}

///
/// Actor that acts as the actual control system and internal engine gateway to the ProcFlow system.
///
pub struct Controller {
    /// Map of root graph nodes that are active in the system.
    pub nodes: HashMap<uuid::Uuid, Aid>,
    /// TX to the Proc Flow engine structure.
    pub sender: Sender<Message>,
    /// RX from the Proc Flow engine structure.
    pub receiver: Receiver<Message>,
    /// Reference to the node library.
    pub catalogue: Arc<Mutex<Catalogue>>,
}

use log::*;

impl Controller {
    ///
    /// Handle messages sent by other actors.
    ///
    pub async fn handle(self, context: Context, message: Message) -> ActorResult<Self> {
        if let Some(msg) = message.content_as::<ControllerCommand>() {
            match &*msg {
                ControllerCommand::GiveMe5 => {
                    let _ = self.sender.send(Message::new(5 as u64));
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
                                                        let _ = self.sender.send(Message::new(
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
                                                        let _ = self.sender.send(Message::new(
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
                                                let _ = self.sender.send(Message::new(
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
                                            let _ = self.sender.send(Message::new(
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
                                    let _ = self.sender.send(Message::new(
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
            }
        }
        Ok(Status::done(self))
    }
}
