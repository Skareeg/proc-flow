use crate::node::*;
use dynamic::*;

use crate::catalogue::*;
use crate::graph::*;

use axiom::prelude::*;

use std::sync::*;

pub struct Engine {
    pub system: ActorSystem,
    pub catalogue: Arc<Mutex<Catalogue>>,
}
