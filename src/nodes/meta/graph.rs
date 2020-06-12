use crate::node::*;
use axiom::actors::*;

use crate::catalogue::*;
use crate::graph::*;

use std::collections::HashMap;

///
/// Represents a user created graph that is loaded from a library.
/// When launched, this will ask the controller to spawn the appropriate nodes for whichever output or recieve that is activated.
///
#[derive(Default)]
pub struct NodeMetaGraphV1 {
    /// Graph version that this graph points to.
    pub graph: Option<GraphRef>,
    /// When the graph has been launched, the in memory representation of the actual graph version.
    pub instance: Option<VersionInfo>,
    /// Actively loaded and running nodes that belong to this graph, keyed by their instance UUID within the graph file.
    pub nodes: HashMap<uuid::Uuid, Aid>,
}

use axiom::prelude::*;
use log::*;

impl Nodeable for NodeMetaGraphV1 {
    fn get_io(&self, _catalogue: &Catalogue) -> (Vec<Pin>, Vec<Pin>) {
        let mut ins = Vec::new();
        let mut outs = Vec::new();
        match self.instance.clone() {
            Some(instance) => {
                trace!("retrieving IO for graph {:?}", &self.graph);
                ins.append(&mut instance.inputs.clone().iter().map(|p| Pin::new_io_basic(p.clone())).collect());
                outs.append(&mut instance.outputs.clone().iter().map(|p| Pin::new_io_basic(p.clone())).collect());
            }
            None => error!("no instance when retrieving IO"),
        }
        (ins, outs)
    }
    fn get_rs(&self, _catalogue: &Catalogue) -> (Vec<Pin>, Vec<Pin>) {
        let mut recvs = Vec::new();
        let mut sends = Vec::new();
        match self.instance.clone() {
            Some(instance) => {
                trace!("retrieving RS for graph {:?}", &self.graph);
                recvs.append(&mut instance.receives.clone().iter().map(|p| Pin::new_rs_basic(p.clone())).collect());
                sends.append(&mut instance.sends.clone().iter().map(|p| Pin::new_rs_basic(p.clone())).collect());
            }
            None => error!("no instance when retrieving IO"),
        }
        (recvs, sends)
    }
    fn compute_output(
        &mut self,
        _node: &mut Node,
        _output_info: PinInfo,
        _context: &Context,
        _parameter: &Option<Message>,
    ) -> Result<Option<Message>, String> {
        todo!()
        // TODO: Load the graph version into memory
    }
    fn handle_receive(
        &mut self,
        _node: &mut Node,
        _context: &Context,
        _receiver: &uuid::Uuid,
        _message: &Option<axiom::message::Message>,
    ) {
        todo!()
    }
}

use std::sync::{Arc, Mutex};

impl NodeMetaGraphV1 {
    pub fn new(controller: Aid, catalogue: Arc<Mutex<Catalogue>>, instance_id: uuid::Uuid) -> Node {
        let process = Self { graph: None, instance: None, nodes: HashMap::new() };
        Node::new(
            NodeInstanceInfo {
                uuid: instance_id,
                data: std::collections::HashMap::new(),
                graph: GraphRef {
                    name: String::from("Graph"),
                    uuid: uuid::Uuid::parse_str("25351e69-098b-4330-9317-37436b03d427").unwrap(),
                    library: uuid::Uuid::parse_str("b0fa443c-20d0-4c2a-acf9-76c63af3cbed").ok(),
                    version: 1,
                },
            },
            Box::new(process),
            catalogue.clone(),
            controller,
        )
    }
}

/// Registers the internal nodes as available graphs to a catalogue.
/// Returns the graphs basic information and the number of versions it has.
pub fn register() -> (GraphInfo, u64) {
    (
        GraphInfo {
            name: String::from("Graph"),
            uuid: uuid::Uuid::parse_str("25351e69-098b-4330-9317-37436b03d427").unwrap(),
            format: 1,
        },
        1,
    )
}

/// Gives back a new internal node object from a given UUID, if it exists.
pub fn create(
    controller: Aid,
    catalogue: Arc<Mutex<Catalogue>>,
    uuid: uuid::Uuid,
    version: u64,
    instance_id: uuid::Uuid,
) -> Option<Node> {
    if uuid == uuid::Uuid::parse_str("25351e69-098b-4330-9317-37436b03d427").unwrap() {
        return match version {
            1 => Some(NodeMetaGraphV1::new(controller, catalogue, instance_id)),
            _ => None,
        };
    }
    None
}
