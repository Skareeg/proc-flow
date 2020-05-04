use crate::node::*;
use axiom::actors::*;

use crate::graph::*;
use crate::catalogue::*;

use std::collections::HashMap;

/// 
/// Represents a user created graph that is loaded from a library.
/// When launched, this will ask the controller to spawn the appropriate nodes for whichever output or recieve that is activated.
/// 
#[derive(Default)]
pub struct NodeMetaGraph {
    /// Graph version that this graph points to.
    pub graph: Option<GraphRef>,
    /// When the graph has been launched, the in memory representation of the actual graph version.
    pub instance: Option<VersionInfo>,
    /// Actively loaded and running nodes that belong to this graph, keyed by their instance UUID within the graph file.
    pub nodes: HashMap<uuid::Uuid, Aid>,
}

use log::*;
use axiom::prelude::*;

impl Nodeable for NodeMetaGraph {
    fn get_io(&self, _catalogue: &Catalogue) -> (Vec<Pin>, Vec<Pin>) {
        let ins = Vec::new();
        let outs = Vec::new();
        match self.graph.clone() {
            Some(graph) => {
                trace!("retrieving IO for graph {:?}", graph);
            },
            None => error!("no graph when retrieving IO")
        }
        (ins, outs)
    }
    fn get_rs(&self, _catalogue: &Catalogue) -> (Vec<Pin>, Vec<Pin>) {
        let recvs = Vec::new();
        let sends = Vec::new();
        (recvs, sends)
    }
    fn compute_output(
        &mut self,
        _node: &mut Node,
        _output_info: PinInfo,
        _context: &Context,
        _parameter: &Option<Message>,
    ) -> Result<Option<Message>, String> { todo!() }
    fn handle_receive(
        &mut self,
        _node: &mut Node,
        _sender: &PinRef,
        _receiver: &PinRef,
        _context: &Context,
        _message: &axiom::message::Message,
    ) { todo!() }
}