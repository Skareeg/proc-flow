use serde::{Deserialize, Serialize};

use dirs::document_dir;
use std::io;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use log::*;

use uuid::*;

use super::graph::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LibraryInfo {
    pub name: String,
    pub uuid: uuid::Uuid,
    pub author: String,
    pub format: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphPathPair {
    pub info: GraphInfo,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Library {
    pub info: LibraryInfo,
    pub path: PathBuf,
    pub graphs: std::collections::HashMap<uuid::Uuid, GraphPathPair>,
}

pub fn get_library_graphs(library_path: PathBuf) -> std::collections::HashMap<uuid::Uuid, GraphPathPair> {
    let mut graphs = std::collections::HashMap::new();
    for entry in walkdir::WalkDir::new(&library_path) {
        match entry {
            Ok(entry) => {
                let entry = entry.path();
                if entry.is_dir() {
                    let graphjson = entry.join(Path::new("graph.json"));
                    if graphjson.is_file() {
                        match std::fs::read_to_string(&graphjson) {
                            Ok(json) => {
                                match serde_json::from_str::<GraphInfo>(&json) {
                                    Ok(info) => {
                                        info!("found graph info {}: {}", entry.display(), info.name);
                                        graphs.insert(info.uuid, GraphPathPair{ path: entry.to_path_buf(), info: info });
                                    },
                                    Err(e) => error!("could not parse {}: {}", graphjson.display(), e)
                                }
                            },
                            Err(e) => error!("could not open {}: {}", graphjson.display(), e)
                        }
                    }
                }
            },
            Err(e) => error!("could not walk directory: {}: {}", library_path.display(), e)
        }
    }
    graphs
}

pub fn get_libraries(libraries_path: PathBuf) -> Vec<Library> {
    let mut libs = Vec::new();
    info!("searching in {} for libraries", libraries_path.display());
    for entry in walkdir::WalkDir::new(&libraries_path) {
        match entry {
            Ok(entry) => {
                let entry = entry.path();
                if entry.is_dir() {
                    let libjson = entry.to_path_buf().join(Path::new("lib.json"));
                    info!("walking {}", entry.display());
                    if libjson.is_file() {
                        match std::fs::read_to_string(&libjson) {
                            Ok(json) => {
                                match serde_json::from_str::<LibraryInfo>(&json) {
                                    Ok(info) => {
                                        let library_path = entry.to_path_buf().clone();
                                        libs.push(Library {
                                            info: info,
                                            path: library_path.clone(),
                                            graphs: get_library_graphs(library_path),
                                        });
                                    },
                                    Err(e) => error!("could not parse {}: {}", libjson.display(), e)
                                }
                            },
                            Err(e) => error!("could not open {}: {}", libjson.display(), e)
                        }
                    }
                }
            },
            Err(e) => error!("could not walk directory: {}: {}", libraries_path.display(), e)
        }
    }
    libs
}