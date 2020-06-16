use axiom::prelude::*;
use crate::graph;

use amethyst::{
    assets::{AssetStorage, Loader, Handle},
    ecs::prelude::*,
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
};
use amethyst::{
    assets::AssetLoaderSystemData,
    core::{
        ecs::{Builder, WorldExt, WriteStorage},
        Transform, TransformBundle,
    },
    renderer::{
        light::{Light, PointLight},
        mtl::{Material, MaterialDefaults},
        palette::{LinSrgba, Srgb},
        plugins::{RenderPbr3D, RenderToWindow},
        rendy::{
            mesh::{Normal, Position, Tangent, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        shape::Shape,
        types::DefaultBackend,
        Mesh, RenderingBundle,
        debug_drawing::*,
    },
    ui::{RenderUi, ToNativeWidget, UiBundle, UiCreator, UiTransformData, UiWidget},
    utils::application_root_dir,
    window::{ScreenDimensions, DisplayConfig},
    Application, GameData, GameDataBuilder, SimpleState, StateData,
};

use crate::node::*;
use axiom::actors::*;

use crate::catalogue::*;
use crate::graph::*;

use std::collections::HashMap;

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
}

use axiom::prelude::*;
use log::*;

use std::sync::*;

use super::camera;
use super::nodes;

struct MyState;

impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let start_width: f64 = 640.0;
        let start_height: f64 = 480.0;
        let mut ui = Arc::new(Mutex::new(conrod_core::UiBuilder::new([start_width, start_height]).build()));
        data.world.insert(ui);
    }
}

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
        todo!()
        // TODO: Load the graph version into memory
    }
    fn handle_receive(
        &mut self,
        _node: &mut Node,
        _context: &Context,
        receiver: &uuid::Uuid,
        _message: &Option<axiom::message::Message>,
    ) {
        match receiver {
            id_edit if id_edit == &uuid::Uuid::parse_str("7c5c2794-eb60-4661-9d25-585e1226233e").unwrap() => {
                amethyst::start_logger(Default::default());
            
                let app_root = application_root_dir().expect("cannot get application_root_dir");
            
                let assets_dir = app_root.join("assets");
                let config_dir = app_root.join("config");
                let display_config_path = config_dir.join("display.ron");

                let mut display_config = DisplayConfig::default();
                display_config.title = String::from("Proc Flow Editor");
                display_config.visibility = true;
                display_config.decorations = true;
                display_config.resizable = true;
            
                let game_data = GameDataBuilder::default()
                .with_bundle(TransformBundle::new()).expect("could not create Amethyst game data")
                .with_bundle(
                        RenderingBundle::<DefaultBackend>::new()
                            .with_plugin(
                                RenderToWindow::from_config(display_config)
                                    .with_clear([0.34, 0.36, 0.52, 1.0]),
                            )
                            //.with_plugin(RenderPbr3D::default()),
                            .with_plugin(RenderUi::default()),
                    ).expect("could not create rendering bundle");
            
                let mut game = Application::new(assets_dir, MyState, game_data).expect("could not create the Amethyst application structure");
                game.run();
            }
            _ => {}
        }
    }
}

use std::sync::{Arc, Mutex};

impl NodeMetaCanvasV1 {
    pub fn new(controller: Aid, catalogue: Arc<Mutex<Catalogue>>, instance_id: uuid::Uuid) -> Node {
        let process = Self { graph: None, instance: None, nodes: HashMap::new() };
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