use crate::node::*;
use dynamic::*;

use crate::graph::*;
use crate::catalogue::*;

use axiom::prelude::*;

#[derive(Default)]
pub struct NodeUtilLogV1 {
}

impl Nodeable for NodeUtilLogV1 {
    fn get_io(&self, catalogue: &Catalogue) -> (std::vec::Vec<Pin>, std::vec::Vec<Pin>) {
        (Vec::new(), Vec::new())
    }

    fn get_rs(&self, catalogue: &Catalogue) -> (std::vec::Vec<Pin>, std::vec::Vec<Pin>) {
        let mut pins_in = Vec::new();
        let mut pins_out = Vec::new();
        pins_in.push(Pin {
            info: PinInfo {
                uuid: uuid::Uuid::new_v4(),
                name: String::from("Info"),
                datatype: String::from("string"),
                dimensions: None,
                expandable: Some(true),
            },
            uuid: uuid::Uuid::new_v4(),
            pin_type: PinType::Input,
            links: Vec::new(),
            value: Default::default(),
        });
        pins_in.push(Pin {
            info: PinInfo {
                uuid: uuid::Uuid::new_v4(),
                name: String::from("Warn"),
                datatype: String::from("string"),
                dimensions: None,
                expandable: Some(true),
            },
            uuid: uuid::Uuid::new_v4(),
            pin_type: PinType::Input,
            links: Vec::new(),
            value: Default::default(),
        });
        pins_in.push(Pin {
            info: PinInfo {
                uuid: uuid::Uuid::new_v4(),
                name: String::from("Error"),
                datatype: String::from("string"),
                dimensions: None,
                expandable: Some(true),
            },
            uuid: uuid::Uuid::new_v4(),
            pin_type: PinType::Input,
            links: Vec::new(),
            value: Default::default(),
        });
        (pins_in, pins_out)
    }

    fn compute_outputs(&self, node: &mut Node, catalogue: &mut Catalogue) -> Result<(), String> {
        Ok(())
    }
    
    fn handle_receives(&self, node: &mut Node, catalogue: &mut Catalogue, context: Context, message: Message) {
    }
}

impl NodeUtilLogV1 {
    pub fn new(catalogue: &Catalogue, x: f32, y: f32) -> Node {
        let node = NodeUtilLogV1 {
        };
        let (inputs, outputs) = node.get_io(catalogue);
        Node {
            info: NodeInfo {
                uuid: uuid::Uuid::new_v4(),
                x,
                y,
                graph: GraphRef {
                    name: String::from("Add"),
                    uuid: uuid::Uuid::parse_str("51be3c2d-7451-4060-a3d3-754e65b6415c").unwrap(),
                    library: String::from("internal"),
                    version: 1,
                }
            },
            inputs,
            outputs,
            process: Dynamic::new(node),
        }
    }
}