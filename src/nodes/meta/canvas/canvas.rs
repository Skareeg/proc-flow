use axiom::prelude::*;
use crate::graph;

// TODO USE ICED FOR NOW.
// TODO AMETHYST FOR 3D Window at the moment.

// First things first, build out voxel and surface nodes.

use iced::button;

#[derive(Default)]
struct Counter {
    value: i32,
    incr_btn: button::State,
    decr_btn: button::State,
}

#[derive(Debug, Clone, Copy)]
pub enum CounterMessage {
    IncrPressed,
    DecrPressed,
}

use iced::{Button, Column, Text, Sandbox, Element};

impl Sandbox for Counter {
    type Message = CounterMessage;
    fn new() -> Self {
        Self::default()
    }
    fn title(&self) -> String {
        String::from("Counter")
    }
    fn view(&mut self) -> Element<CounterMessage> {
        Column::new()
            .push(
                Button::new(&mut self.incr_btn, Text::new("+")).on_press(CounterMessage::IncrPressed),
            )
            .push(
                Text::new(self.value.to_string()).size(50),
            )
            .push(
                Button::new(&mut self.decr_btn, Text::new("-")).on_press(CounterMessage::DecrPressed),
            ).into()
    }
    fn update(&mut self, message: CounterMessage) {
        match message {
            CounterMessage::IncrPressed => self.value += 1,
            CounterMessage::DecrPressed => self.value -= 1,
        }
    }
}

use crate::node::*;

use crate::catalogue::*;
use crate::graph::*;

use std::collections::HashMap;
use crossbeam::{Receiver, Sender};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum CanvasMessage {
    Exit,
}

///
/// Represents a node-editing canvas window.
/// When launched, this will open an editor for creating and updating graphs and their versions.
///
#[derive(Default)]
pub struct NodeMetaCanvasV1 {
    /// Graph version that this canvas is currently editing.
    pub graph: Option<GraphRef>,
    /// When the graph has been launched, the in memory representation of the actual graph version that is being edited.
    pub instance: Option<VersionInfo>,
    /// Actively loaded and running nodes that belong to this canvas, keyed by their instance UUID within the graph file.
    pub nodes: HashMap<uuid::Uuid, Aid>,
    /// Comms to the actual editor window.
    /// Receiver is not necessary as the window will send messages directly to the node actor.
    pub send_to_editor_window: Option<Sender<CanvasMessage>>,
}
use log::*;

use std::sync::*;

use super::camera;
use super::nodes;

impl Nodeable for NodeMetaCanvasV1 {
    fn get_io(&self, _catalogue: &Catalogue) -> (Vec<Pin>, Vec<Pin>) {
        let mut ins = Vec::new();
        let mut outs = Vec::new();
        ins.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Graph Folder Path"),
            uuid::Uuid::parse_str("3db2a9ea-9c77-4b4f-b18b-e5418f0e1f4a").unwrap(),
            String::from("string"),
        )));
        ins.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Graph Version"),
            uuid::Uuid::parse_str("af58cc69-0db9-4a4a-b715-774bf9e7faa6").unwrap(),
            String::from("int"),
        )));
        outs.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Modified Graph Folder Path"),
            uuid::Uuid::parse_str("44a986b1-dc09-45d9-ab65-e2c0c7b6f5ce").unwrap(),
            String::from("string"),
        )));
        outs.push(Pin::new_io_basic(PinInfo::new_basic(
            String::from("Modified Graph Version"),
            uuid::Uuid::parse_str("fdbc0064-6aa5-41f5-85f8-be17659821e7").unwrap(),
            String::from("int"),
        )));
        (ins, outs)
    }
    fn get_rs(&self, _catalogue: &Catalogue) -> (Vec<Pin>, Vec<Pin>) {
        let mut recvs = Vec::new();
        let mut sends = Vec::new();
        recvs.push(Pin::new_rs_basic(PinInfo::new_basic(
            String::from("Edit"),
            uuid::Uuid::parse_str("7c5c2794-eb60-4661-9d25-585e1226233e").unwrap(),
            String::from("none"),
        )));
        recvs.push(Pin::new_rs_basic(PinInfo::new_basic(
            String::from("Save"),
            uuid::Uuid::parse_str("c0cf0e78-4171-4fbc-ad19-fd6bc372b69e").unwrap(),
            String::from("none"),
        )));
        (recvs, sends)
    }
    fn compute_output(
        &mut self,
        _node: &mut Node,
        _output_info: PinInfo,
        _context: &Context,
        _parameter: &Option<Message>,
    ) -> Result<Option<Message>, String> {
        //todo!()
        Ok(None)
        // TODO: Load the graph version into memory
    }
    fn handle_receive(
        &mut self,
        _node: &mut Node,
        context: &Context,
        receiver: &uuid::Uuid,
        _message: &Option<axiom::message::Message>,
    ) {
        info!("canvas recv");
        match receiver {
            id_edit if id_edit == &uuid::Uuid::parse_str("7c5c2794-eb60-4661-9d25-585e1226233e").unwrap() => {
                info!("canvas edit");
                //Counter::run(iced::Settings::default());
                let (send_to_editor_window, recv_from_editor_window) = crossbeam::unbounded();
                self.send_to_editor_window = Some(send_to_editor_window);
                let node_aid = context.aid.clone();
                std::thread::spawn(|| super::editor::run_canvas_editor(node_aid, recv_from_editor_window));
            }
            _ => {}
        }
    }
    // Handle messages, most likely generated from the actual window thread.
    fn handle_message(
        &mut self,
        node: &mut Node,
        _context: &Context,
        message: &Message
    ) {
        if let Some(msg) = message.content_as::<CanvasMessage>() {
            match &*msg {
                CanvasMessage::Exit => {
                    let _ = node.controller.send_new(crate::engine::ControllerCommand::StopWaitingForNewMessages);
                }
                _ => {}
            }
        }
    }
}

use std::sync::{Arc, Mutex};

impl NodeMetaCanvasV1 {
    pub fn new(controller: Aid, catalogue: Arc<Mutex<Catalogue>>, instance_id: uuid::Uuid) -> Node {
        let process = Self { graph: None, instance: None, nodes: HashMap::new(), send_to_editor_window: None };
        Node::new(
            NodeInstanceInfo {
                uuid: instance_id,
                data: std::collections::HashMap::new(),
                graph: GraphRef {
                    name: String::from("Canvas"),
                    uuid: uuid::Uuid::parse_str("a795c3e9-0a2f-48bf-a9e2-03378e8e59b8").unwrap(),
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
            name: String::from("Canvas"),
            uuid: uuid::Uuid::parse_str("a795c3e9-0a2f-48bf-a9e2-03378e8e59b8").unwrap(),
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
    if uuid == uuid::Uuid::parse_str("a795c3e9-0a2f-48bf-a9e2-03378e8e59b8").unwrap() {
        return match version {
            1 => Some(NodeMetaCanvasV1::new(controller, catalogue, instance_id)),
            _ => None,
        };
    }
    None
}

// Test actor TCPClusterMgr one actor to another systems actor via Aid.

// Create this entire canvas editor as a node that launches a window.
// Takes initial graph location and version as input.
// Blank input is a new blank graph version.
// Outputs new graph location and version if changed.
// Must read in graph version file and build an internal representation of the graph version graph.
// Use conrod for shape drawing to vertex buffer.
// Draw buffer via shader to render nodes to screen.
// Keep track of camera location and zoom.
// Accept input to move nodes, wire nodes together, and create and delete new nodes.
// Spacebar searchbox for node instance creation. Box with text entry. Type brings button search results below. Assume latest version of graph.