use specs::prelude::*;
use crate::graph::*;

pub struct NodeComponent {
    pub graph_ref: GraphRef,
    pub instance: Option<VersionInfo>,
}

impl Component for NodeComponent {
    type Storage = DenseVecStorage<Self>;
}