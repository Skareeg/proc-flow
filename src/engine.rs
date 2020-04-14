use crate::node::*;
use dynamic::*;

use crate::catalogue::*;
use crate::graph::*;

use axiom::prelude::*;

use std::sync::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum EngineCommand {
}

/// 
/// Proc Flow instance and node scheduler.
/// 
pub struct Engine {
    /// The actor system in which to run nodes.
    pub system: ActorSystem,
    pub catalogue: Arc<Mutex<Catalogue>>,
    pub send: crossbeam::Sender<Message>,
    pub recv: crossbeam::Receiver<Message>,
    pub nodes: HashMap<uuid::Uuid, Aid>,
}

impl Engine {
    pub fn new() -> Self {
        let system = ActorSystem::create(ActorSystemConfig::default());
        let catalogue = Arc::new(Mutex::new(Catalogue::new()));
        let (send, recv) = crossbeam::unbounded();
        let nodes = HashMap::new();
        Self {
            system,
            catalogue,
            send,
            recv,
            nodes
        }
    }
    pub fn boot_graph(&mut self, graph: GraphRef) {
    }
    pub fn boot_cluster(&mut self, port: u64) {
    }
}

pub struct Controller {
}