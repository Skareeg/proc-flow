use crate::node::*;
use dynamic::*;
use axiom::actors::*;

use crate::graph::*;
use crate::catalogue::*;

#[derive(Default)]
pub struct NodeMetaGraph {
    pub graph: Option<GraphRef>,
}

use log::*;
use axiom::prelude::*;

impl Nodeable for NodeMetaGraph {
    fn get_io(&self, catalogue: &Catalogue) -> (Vec<Pin>, Vec<Pin>) {
        let mut ins = Vec::new();
        let mut outs = Vec::new();
        match self.graph.clone() {
            Some(graph) => {
                trace!("retrieving IO for graph {:?}", graph);
            },
            None => error!("no graph when retrieving IO")
        }
        (ins, outs)
    }
    fn get_rs(&self, catalogue: &Catalogue) -> (Vec<Pin>, Vec<Pin>) {
        let mut recvs = Vec::new();
        let mut sends = Vec::new();
        (recvs, sends)
    }
    fn compute_output(
        &mut self,
        node: &mut Node,
        output_info: PinInfo,
        context: &Context,
        parameter: &Option<Message>,
    ) -> Result<Option<Message>, String> { todo!() }
    fn handle_receive(
        &mut self,
        node: &mut Node,
        sender: &PinRef,
        receiver: &PinRef,
        context: &Context,
        message: &axiom::message::Message,
    ) { todo!() }
}