///
/// This holds the necessary information to execute Proc Flow graphs.
/// 
pub struct GraphManager {
    pub catalogue: catalogue::Catalogue,
    pub current_graph: graph::GraphRef,
    pub system: axiom::system::ActorSystem,
}