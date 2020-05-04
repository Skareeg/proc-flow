pub mod log;

use crate::graph::*;

/// Registers the internal nodes as available graphs to a catalogue.
pub fn register() -> Vec<(GraphInfo, u64)> {
    let mut nodes = Vec::new();
    nodes.push(log::register());
    nodes
}

use axiom::prelude::*;
use crate::node::*;
use crate::catalogue::*;
use std::sync::{Arc, Mutex};

/// Gives back a new internal node object from a given UUID, if it exists.
pub fn create(controller: Aid, catalogue: Arc<Mutex<Catalogue>>, uuid: uuid::Uuid, version: u64, instance_id: uuid::Uuid) -> Option<Node> {
    log::create(controller, catalogue, uuid, version, instance_id)
}