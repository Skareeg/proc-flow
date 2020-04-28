//pub mod math;
//pub mod string;
pub mod util;
pub mod meta;

use crate::graph::*;

/// Registers the internal nodes as available graphs to a catalogue.
pub fn register() -> Vec<(GraphInfo, u64)> {
    let mut nodes = Vec::new();
    nodes.append(&mut util::register());
    nodes
}

use crate::node::*;
use crate::catalogue::*;
use std::sync::{Arc, Mutex, MutexGuard};

/// Gives back a new internal node object from a given UUID, if it exists.
pub fn create(catalogue: Arc<Mutex<Catalogue>>, x: f32, y: f32, uuid: uuid::Uuid, version: u64) -> Option<Node> {
    util::create(catalogue, x, y, uuid, version)
}