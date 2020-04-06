use crate::node::*;
use dynamic::*;

use crate::graph::*;
use crate::catalogue::*;

#[derive(Default)]
pub struct NodeMathBasicAddU8v1 {
    pub result: u8,
}

impl Nodeable for NodeMathBasicAddU8v1 {
    fn get_io(&self, catalogue: &Catalogue) -> (std::vec::Vec<Pin>, std::vec::Vec<Pin>) {
        let mut pins_in = Vec::new();
        let mut pins_out = Vec::new();
        pins_in.push(Pin {
            info: PinInfo {
                uuid: uuid::Uuid::new_v4(),
                name: "Numbers".to_owned(),
                datatype: "u8".to_owned(),
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
                name: "B".to_owned(),
                datatype: "u8".to_owned(),
                dimensions: None,
                expandable: None,
            },
            uuid: uuid::Uuid::new_v4(),
            pin_type: PinType::Input,
            links: Vec::new(),
            value: Default::default(),
        });
        pins_out.push(Pin {
            info: PinInfo {
                uuid: uuid::Uuid::new_v4(),
                name: "Result".to_owned(),
                datatype: "u16".to_owned(),
                dimensions: None,
                expandable: None,
            },
            uuid: uuid::Uuid::new_v4(),
            pin_type: PinType::Output,
            links: Vec::new(),
            value: Default::default(),
        });
        (pins_in, pins_out)
    }

    fn compute_outputs(&self, node: &mut Node, catalogue: &mut Catalogue) -> Result<(), String> {
        Ok(())
    }
}

impl NodeMathBasicAddU8v1 {
    pub fn new(catalogue: &Catalogue, x: f32, y: f32) -> Node {
        let node = NodeMathBasicAddU8v1 {
            result: 0,
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