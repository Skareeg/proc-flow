use crate::node::*;
use dynamic::*;

use crate::catalogue::*;
use crate::graph::*;

use axiom::prelude::*;

use std::sync::*;

/// 
/// Proc Flow instance and node scheduler.
/// 
pub struct Engine {
    /// The actor system in which to run nodes.
    pub system: ActorSystem,
    pub catalogue: Arc<Mutex<Catalogue>>,
}
