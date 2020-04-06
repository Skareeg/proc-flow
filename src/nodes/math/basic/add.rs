use crate::node::*;
use dynamic::*;

use crate::graph::*;
use crate::catalogue::*;

#[derive(Default)]
pub struct NodeMathBasicAddU8 {
    pub result: u8,
}

impl PinInputtable for NodeMathBasicAddU8 {
    fn get_inputs(&self, catalogue: &Catalogue) -> std::vec::Vec<Pin> {
        let mut pins = Vec::new();
        pins.push(Pin {
            info: PinInfo {
                uuid: uuid::Uuid::new_v4(),
                name: "B".to_owned(),
                datatype: "u8".to_owned(),
                dimensions: None,
            },
            uuid: uuid::Uuid::new_v4(),
            pin_type: PinType::Input,
            links: Vec::new(),
            value: Default::default(),
        });
        pins.push(Pin {
            info: PinInfo {
                uuid: uuid::Uuid::new_v4(),
                name: "B".to_owned(),
                datatype: "u8".to_owned(),
                dimensions: None,
            },
            uuid: uuid::Uuid::new_v4(),
            pin_type: PinType::Input,
            links: Vec::new(),
            value: Default::default(),
        });
        pins
    }
}

impl PinOutputtable for NodeMathBasicAddU8 {
    fn get_outputs(&self, catalogue: &Catalogue) -> std::vec::Vec<Pin> {
        let mut pins = Vec::new();
        pins.push(Pin {
            info: PinInfo {
                uuid: uuid::Uuid::new_v4(),
                name: "Result".to_owned(),
                datatype: "u16".to_owned(),
                dimensions: None,
            },
            uuid: uuid::Uuid::new_v4(),
            pin_type: PinType::Output,
            links: Vec::new(),
            value: Default::default(),
        });
        pins
    }
}

impl NodeProcess for NodeMathBasicAddU8 {
    fn compute_outputs(&self, node: &mut Node, catalogue: &mut Catalogue) -> Result<(), String> {
        Ok(())
    }
}