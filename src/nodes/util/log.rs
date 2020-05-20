use crate::node::*;

use crate::catalogue::*;
use crate::graph::*;

use axiom::prelude::*;

#[derive(Default)]
pub struct NodeUtilLogV1 {}

use log::*;

impl Nodeable for NodeUtilLogV1 {
    fn get_io(&self, _catalogue: &Catalogue) -> (std::vec::Vec<Pin>, std::vec::Vec<Pin>) {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        inputs.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Info"),
            uuid::Uuid::parse_str("5e6ab872-5cca-4e01-8dbb-2df843102dc0").unwrap(),
            String::from("string"),
        )));
        inputs.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Warn"),
            uuid::Uuid::parse_str("2916bcb7-2943-4426-8af4-292bd8b1f417").unwrap(),
            String::from("string"),
        )));
        inputs.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Error"),
            uuid::Uuid::parse_str("f39a4e33-32f3-485f-b634-e539c98dbe94").unwrap(),
            String::from("string"),
        )));
        outputs.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Info"),
            uuid::Uuid::parse_str("44a986b1-dc09-45d9-ab65-e2c0c7b6f5ce").unwrap(),
            String::from("string"),
        )));
        outputs.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Warn"),
            uuid::Uuid::parse_str("d792d30a-0986-4f8c-bf6d-5fd0f4ac3d05").unwrap(),
            String::from("string"),
        )));
        outputs.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Error"),
            uuid::Uuid::parse_str("2af8bac9-9d56-4f6f-b997-68b05d1f3e55").unwrap(),
            String::from("string"),
        )));
        (inputs, outputs)
    }

    fn get_rs(&self, _catalogue: &Catalogue) -> (std::vec::Vec<Pin>, std::vec::Vec<Pin>) {
        let mut recvs = Vec::new();
        let mut sends = Vec::new();
        recvs.push(Pin::new_rs_basic(PinInfo::new_basic(
            String::from("Info"),
            uuid::Uuid::parse_str("6b9c6c69-13e8-473a-ac47-818fcdf6d7bd").unwrap(),
            String::from("string"),
        )));
        recvs.push(Pin::new_rs_basic(PinInfo::new_basic(
            String::from("Warn"),
            uuid::Uuid::parse_str("4eb1bc59-ca1b-4754-be49-0ad13f86421a").unwrap(),
            String::from("string"),
        )));
        recvs.push(Pin::new_rs_basic(PinInfo::new_basic(
            String::from("Error"),
            uuid::Uuid::parse_str("3f66f874-b785-4444-b7c6-5007052b531c").unwrap(),
            String::from("string"),
        )));
        recvs.push(Pin::new_rs_basic(PinInfo::new_basic(
            String::from("Log and Pass Through"),
            uuid::Uuid::parse_str("bccf1a26-793d-4c80-ad25-be110c4dc1d7").unwrap(),
            String::from("string"),
        )));
        sends.push(Pin::new_rs_basic(PinInfo::new_basic(
            String::from("Info"),
            uuid::Uuid::parse_str("dfc26f11-fa2b-4667-aad3-456edbdd9c84").unwrap(),
            String::from("string"),
        )));
        sends.push(Pin::new_rs_basic(PinInfo::new_basic(
            String::from("Warn"),
            uuid::Uuid::parse_str("3982006c-9e32-4e59-a544-58bc9a367daf").unwrap(),
            String::from("string"),
        )));
        sends.push(Pin::new_rs_basic(PinInfo::new_basic(
            String::from("Error"),
            uuid::Uuid::parse_str("ab04b49d-ff65-44c6-a70b-8546ecdbc5ba").unwrap(),
            String::from("string"),
        )));
        (recvs, sends)
    }

    fn compute_output(
        &mut self,
        node: &mut Node,
        output_info: PinInfo,
        _context: &Context,
        _parameter: &Option<Message>,
    ) -> Result<Option<Message>, String> {
        match node.outputs.get_mut(&output_info.uuid) {
            Some(output) => match output.info.uuid {
                id_info
                    if id_info
                        == uuid::Uuid::parse_str("44a986b1-dc09-45d9-ab65-e2c0c7b6f5ce")
                            .unwrap() =>
                {
                    let input = node
                        .inputs
                        .get(
                            &uuid::Uuid::parse_str("5e6ab872-5cca-4e01-8dbb-2df843102dc0").unwrap(),
                        )
                        .expect("could not find corresponding log input");
                    if let Some(value) = input.value.clone() {
                        if let Some(value) = value.content_as::<String>() {
                            info!("{}", value);
                        }
                    }
                    Ok(input.value.clone())
                }
                id_warn
                    if id_warn
                        == uuid::Uuid::parse_str("d792d30a-0986-4f8c-bf6d-5fd0f4ac3d05")
                            .unwrap() =>
                {
                    let input = node
                        .inputs
                        .get(
                            &uuid::Uuid::parse_str("2916bcb7-2943-4426-8af4-292bd8b1f417").unwrap(),
                        )
                        .expect("could not find corresponding log input");
                    if let Some(value) = input.value.clone() {
                        if let Some(value) = value.content_as::<String>() {
                            warn!("{}", value);
                        }
                    }
                    Ok(input.value.clone())
                }
                id_error
                    if id_error
                        == uuid::Uuid::parse_str("2af8bac9-9d56-4f6f-b997-68b05d1f3e55")
                            .unwrap() =>
                {
                    let input = node
                        .inputs
                        .get(
                            &uuid::Uuid::parse_str("f39a4e33-32f3-485f-b634-e539c98dbe94").unwrap(),
                        )
                        .expect("could not find corresponding log input");
                    if let Some(value) = input.value.clone() {
                        if let Some(value) = value.content_as::<String>() {
                            error!("{}", value);
                        }
                    }
                    Ok(input.value.clone())
                }
                _ => panic!("could not find corresponding input uuid for log node"),
            },
            None => Err(format!("output pin with uuid {}", output_info.uuid)),
        }
    }
    fn handle_receive(
        &mut self,
        _node: &mut Node,
        _sender: &PinRef,
        _receiver: &PinRef,
        _context: &Context,
        _message: &Message,
    ) {
        todo!()
    }
}

use std::sync::{Arc, Mutex};

impl NodeUtilLogV1 {
    pub fn new(controller: Aid, catalogue: Arc<Mutex<Catalogue>>, instance_id: uuid::Uuid) -> Node {
        let process = Self {};
        Node::new(
            NodeInstanceInfo {
                uuid: instance_id,
                data: None,
                graph: GraphRef {
                    name: String::from("Log"),
                    uuid: uuid::Uuid::parse_str("fd41d8ef-d10f-4499-8a90-35b73d8ff246").unwrap(),
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
            name: String::from("Log"),
            uuid: uuid::Uuid::parse_str("fd41d8ef-d10f-4499-8a90-35b73d8ff246").unwrap(),
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
    if uuid == uuid::Uuid::parse_str("fd41d8ef-d10f-4499-8a90-35b73d8ff246").unwrap() {
        return match version {
            1 => Some(NodeUtilLogV1::new(controller, catalogue, instance_id)),
            _ => None,
        };
    }
    None
}
