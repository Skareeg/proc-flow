use serde::{Deserialize, Serialize};

///
/// Information about a pin.
/// 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PinInfo {
    pub name: String,
    pub uuid: uuid::Uuid,
    pub datatype: String,
    pub dimensions: Option<u16>,
    pub expandable: Option<bool>,
}

///
/// Reference to an external graph, or this one.
/// 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphRef {
    pub name: String,
    pub uuid: uuid::Uuid,
    pub library: String,
    pub version: u64,
}

///
/// Information for a given node.
/// 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeInfo {
    pub uuid: uuid::Uuid,
    pub x: f32,
    pub y: f32,
    pub graph: GraphRef,
}

///
/// Reference to a pin within the graph.
/// 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PinRef {
    pub node: uuid::Uuid,
    pub pin: uuid::Uuid,
    pub index: Option<u16>,
    pub value: Option<String>,
}

///
/// References to two connected pins.
/// 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectionInfo {
    pub receives: Option<PinRef>,
    pub sends: Option<PinRef>,
    pub output: Option<PinRef>,
    pub input: Option<PinRef>,
}

///
/// Information about a given version of the graph.
/// 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VersionInfo {
    pub format: u16,
    pub receives: Vec<PinInfo>,
    pub sends: Vec<PinInfo>,
    pub inputs: Vec<PinInfo>,
    pub outputs: Vec<PinInfo>,
    pub nodes: Vec<NodeInfo>,
    pub connections: Vec<ConnectionInfo>,
}

///
/// Information about a graph as a whole, regardless of version.
/// 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphInfo {
    pub name: String,
    pub uuid: uuid::Uuid,
    pub format: u16,
}

use super::library::*;
use std::path::{Path, PathBuf};

use log::*;

pub fn get_graph_version_from_library(lib: &Library, id: uuid::Uuid, version: u64) -> Option<VersionInfo> {
    match lib.graphs.get(&id) {
        Some(pair) => {
            let graph_path = &pair.path;
            let graph_info = &pair.info;
            let version_path = graph_path.join(PathBuf::from(format!("{}", version)));
            match version_path.is_dir() {
                true => {
                    let version_json = version_path.join(PathBuf::from("version.json"));
                    match version_json.is_file() {
                        true => match std::fs::read_to_string(&version_json) {
                            Ok(json) => {
                                match serde_json::from_str::<VersionInfo>(&json) {
                                    Ok(info) => {
                                        Some(info)
                                    },
                                    Err(e) => {
                                        error!("could not read {} version json: {}", version_json.display(), e); None
                                    }
                                }
                            },
                            Err(e) => {
                                error!("could not read {} version file: {}", version_json.display(), e); None
                            }
                        },
                        false => { error!("version.json file does not exist in the {} directory", version_path.display()); None }
                    }
                },
                false => { error!("version directory {} does not exist for {} graph with id of {}", version_path.display(), graph_info.name, graph_info.uuid); None }
            }
        },
        _ => { info!("graph with id of {} does not exist in the {} library with id of {}", id, lib.info.name, lib.info.uuid); None }
    }
}