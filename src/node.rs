use serde::{Serialize, Deserialize};
use axiom::prelude::*;
use dynamic::*;
use crate::graph::*;
use crate::catalogue::*;

extern crate crossbeam;

pub trait Named {
    fn get_name(&self) -> &str;
}

pub trait Nodeable {
    fn get_io(&self, catalogue: &Catalogue) -> (std::vec::Vec<Pin>, std::vec::Vec<Pin>);
    fn compute_outputs(&self, node: &mut Node, catalogue: &mut Catalogue) -> Result<(), String>;
}

pub enum PinType {
    Input,
    Output,
}

pub struct Pin {
    pub info: PinInfo,
    pub uuid: uuid::Uuid,
    pub pin_type: PinType,
    pub links: Vec<PinRef>,
    pub value: Option<Box<Dynamic>>,
}

impl Named for Pin {
    fn get_name(&self) -> &str {
        &*self.info.name
    }
}

pub struct Node {
    pub info: NodeInfo,
    pub inputs: std::vec::Vec<Pin>,
    pub outputs: std::vec::Vec<Pin>,
    pub process: Box<Dynamic>,
}

impl Named for Node {
    fn get_name(&self) -> &str {
        &*self.info.graph.name
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeCommand {
    /// Executes a node, creating the output values.
    /// Will send progress back to caller.
    /// Aid is the commander.
    ComputeOutputs(Aid),
    /// Requests the progress of a node.
    /// Aid is the requestor.
    RequestProgress(Aid),
    /// Provides the progress of a given node.
    /// Aid is the progressing node, and f32 is the progress.
    UpdateProgress(Aid, f32),
}

impl Node {
    pub async fn handle(self, context: Context, message: Message) -> ActorResult<Self> {
        if let Some(msg) = message.content_as::<NodeCommand>() {
            match &*msg {
                NodeCommand::ComputeOutputs(commander) => { Ok(Status::done(self)) },
                NodeCommand::RequestProgress(requestor) => { Ok(Status::done(self)) },
                NodeCommand::UpdateProgress(progressor, progress) => { Ok(Status::done(self)) },
            }
        } else {
            Ok(Status::done(self))
        }
    }
}