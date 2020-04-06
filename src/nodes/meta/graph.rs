use crate::node::*;
use dynamic::*;

use crate::graph::*;
use crate::catalogue::*;

#[derive(Default)]
pub struct NodeMetaGraph {
    pub graph: Option<GraphRef>,
}

impl Nodeable for NodeMetaGraph {
    fn get_io(&self, catalogue: &Catalogue) -> (std::vec::Vec<Pin>, std::vec::Vec<Pin>) {
        let mut pins_in = Vec::new();
        let mut pins_out = Vec::new();
        pins_in.push(Pin {
            info: PinInfo {
                uuid: uuid::Uuid::new_v4(),
                name: "A".to_owned(),
                datatype: "u8".to_owned(),
                dimensions: None,
                expandable: None,
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