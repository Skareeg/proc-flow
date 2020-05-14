//pub mod math;
//pub mod string;
pub mod meta;
pub mod util;

use crate::graph::*;

/// Registers the internal nodes as available graphs to a catalogue.
pub fn register() -> Vec<(GraphInfo, u64)> {
    let mut nodes = Vec::new();
    nodes.append(&mut util::register());
    nodes
}

use crate::catalogue::*;
use crate::node::*;
use axiom::prelude::*;
use std::sync::{Arc, Mutex};

/// Gives back a new internal node object from a given UUID, if it exists.
pub fn create(
    controller: Aid,
    catalogue: Arc<Mutex<Catalogue>>,
    uuid: uuid::Uuid,
    version: u64,
    instance_id: uuid::Uuid,
) -> Option<Node> {
    util::create(controller, catalogue, uuid, version, instance_id)
}
