// use crate::node::*;

use crate::catalogue::*;
// use crate::graph::*;

use axiom::prelude::*;

use std::sync::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crossbeam::{Sender, Receiver};

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

        // Create and spawn the controller.
        let controller_state = Controller {
            nodes,
            sender: send.clone(),
            receiver: recv.clone(),
            catalogue: catalogue.clone()
        };
        let controller = system.spawn().name("controller").with(controller_state, Controller::handle).expect("could not create engine controller");

        // Return newly contructed engine.
        Self {
            system,
            catalogue,
            send,
            recv,
            controller
        }
    }
    pub fn test_5(&mut self) {
        let _ = self.controller.send_new_after(ControllerCommand::GiveMe5, std::time::Duration::from_secs(5));
        info!("Prepared to send a giveme5 request");
        if let Some(num) = self.recv.recv().unwrap().content_as::<u64>() {
            info!("Received the number {}", num);
        }
    }
    pub fn boot_graph(&mut self, id: uuid::Uuid, version: u64) {
        let cat = self.catalogue.lock().expect("can't lock the catalogue mutex when attempting to call boot_graph on engine.");
        let ver = cat.get_graph_version(id, version);
        match ver {
            Some(ver) => {},
            None => {}
        }
    }
    pub fn boot_cluster(&mut self, port: u64) {
    }
}

/// 
/// Each possible command from the engine to the controller.
/// 
#[derive(Serialize, Deserialize)]
enum ControllerCommand {
    /// Test command to ensure that the controller is functional.
    /// TODO: Remove when the system is working.
    GiveMe5,
    /// Boots up a graph instance to gather IO from.
    /// UUID is the graph's UUID in the catalogue.
    /// Number is the version number to load.
    BootGraph(uuid::Uuid, u64)
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
    pub async fn handle(mut self, context: Context, message: Message) -> ActorResult<Self> {
        if let Some(msg) = message.content_as::<ControllerCommand>() {
            match &*msg {
                ControllerCommand::GiveMe5 => {
                    let _ = self.sender.send(Message::new(5 as u64));
                },
                ControllerCommand::BootGraph(id, version) => {
                    info!("boot sequence initiated for graph {} version {}", id, version);
                }
            }
        }
        Ok(Status::done(self))
    }
}