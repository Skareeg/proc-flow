use serde::{Deserialize, Serialize};

///
/// Information about a pin.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PinInfo {
    /// The name of this pin.
    pub name: String,
    /// The id of this pin in regards to the graph.
    pub uuid: uuid::Uuid,
    /// The pin's datatype.
    pub datatype: String,
    /// Whether or not the user can type a constant directly into this input.
    pub valuable: Option<bool>,
    /// If it exists and is greater than 1, how many dimensions the matrix is.
    pub dimensions: Option<u16>,
    /// Whether or not this pin is expandable, meaning many pins represent this one input as an array.
    pub expandable: Option<bool>,
}

impl PinInfo {
    pub fn new_basic(name: String, uuid: uuid::Uuid, datatype: String) -> Self {
        Self {
            name,
            uuid,
            datatype,
            valuable: None,
            dimensions: None,
            expandable: None,
        }
    }
    pub fn new_extended(
        name: String,
        uuid: uuid::Uuid,
        datatype: String,
        valuable: Option<bool>,
        dimensions: Option<u16>,
        expandable: Option<bool>,
    ) -> Self {
        Self {
            name,
            uuid,
            datatype,
            valuable,
            dimensions,
            expandable,
        }
    }
}

///
/// Reference to an external graph, or this one.
/// Blank library means "this" one.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphRef {
    pub name: String,
    pub uuid: uuid::Uuid,
    pub library: Option<uuid::Uuid>,
    pub version: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Datum {
    pub name: String,
    pub value: serde_json::Value,
}

///
/// Information for a given node.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeInfo {
    /// The id of this individual instance of the node.
    pub uuid: uuid::Uuid,
    /// The horizontal position of this instance in the graph.
    pub x: f32,
    /// The vertical position of this instance in the graph.
    pub y: f32,
    /// Data variables used by the node to store data that is not held on any inputs, including large array data, matrix data, and a string or many strings pointing to relative file paths or not-recommended absolute file paths of data files.
    /// Basically just whatever the node needs to hold.
    pub data: Option<Vec<Datum>>,
    /// The reference to the graph that this node instance represents.
    pub graph: GraphRef,
}

///
/// Reference to a pin within the graph.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PinRef {
    /// The node this pin references.
    pub node: uuid::Uuid,
    /// The pin this reference targets.
    pub pin: Option<uuid::Uuid>,
    /// A built-in property of the node this reference targets.
    pub property: Option<String>,
    /// Whether or not this particular output has been designated in the graph to cache its value.
    /// This defaults to true for both inputs and outputs.
    pub cache: Option<bool>,
    /// A direct value input from the user.
    pub value: Option<serde_json::Value>,
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
use std::path::PathBuf;

use log::*;

pub fn get_graph_version_from_library(
    lib: &Library,
    id: uuid::Uuid,
    version: u64,
) -> Option<VersionInfo> {
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
                            Ok(json) => match serde_json::from_str::<VersionInfo>(&json) {
                                Ok(info) => Some(info),
                                Err(e) => {
                                    error!(
                                        "could not read {} version json: {}",
                                        version_json.display(),
                                        e
                                    );
                                    None
                                }
                            },
                            Err(e) => {
                                error!(
                                    "could not read {} version file: {}",
                                    version_json.display(),
                                    e
                                );
                                None
                            }
                        },
                        false => {
                            error!(
                                "version.json file does not exist in the {} directory",
                                version_path.display()
                            );
                            None
                        }
                    }
                }
                false => {
                    error!(
                        "version directory {} does not exist for {} graph with id of {}",
                        version_path.display(),
                        graph_info.name,
                        graph_info.uuid
                    );
                    None
                }
            }
        }
        _ => {
            info!(
                "graph with id of {} does not exist in the {} library with id of {}",
                id, lib.info.name, lib.info.uuid
            );
            None
        }
    }
}

pub fn has_graph_version_from_library(lib: &Library, id: uuid::Uuid, version: u64) -> bool {
    match lib.graphs.get(&id) {
        Some(pair) => {
            let graph_path = &pair.path;
            let graph_info = &pair.info;
            let version_path = graph_path.join(PathBuf::from(format!("{}", version)));
            match version_path.is_dir() {
                true => {
                    let version_json = version_path.join(PathBuf::from("version.json"));
                    match version_json.is_file() {
                        true => true,
                        false => {
                            error!(
                                "version file {} does not exist for {} graph with id of {}",
                                version_json.display(),
                                graph_info.name,
                                graph_info.uuid
                            );
                            false
                        }
                    }
                }
                false => {
                    error!(
                        "version directory {} does not exist for {} graph with id of {}",
                        version_path.display(),
                        graph_info.name,
                        graph_info.uuid
                    );
                    false
                }
            }
        }
        _ => {
            error!(
                "graph with id of {} does not exist in the {} library with id of {}",
                id, lib.info.name, lib.info.uuid
            );
            false
        }
    }
}
