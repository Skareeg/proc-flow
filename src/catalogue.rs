use serde::{Deserialize, Serialize};

use dirs::document_dir;
use std::path::PathBuf;

use log::*;

use super::graph::*;
use super::library::*;

fn load_libraries() -> Vec<Library> {
    let mut libs = Vec::new();
    let mut internal = Library {
        info: LibraryInfo {
            name: String::from("internal"),
            uuid: uuid::Uuid::parse_str("b0fa443c-20d0-4c2a-acf9-76c63af3cbed").unwrap(),
            author: String::from("Proc Flow Internal"),
            format: 1,
        },
        path: PathBuf::default(),
        graphs: std::collections::HashMap::new(),
    };
    for (info, versions) in super::nodes::register() {
        internal.graphs.insert(
            info.uuid.clone(),
            LibraryGraphInfo {
                info: info.clone(),
                path: PathBuf::default(),
                versions: versions.clone(),
            },
        );
        info!(
            "added graph {} : {} with {} versions to internal library",
            info.uuid.clone(),
            info.name.clone(),
            versions.clone()
        );
    }
    info!("added internal libraries to catalogue");
    let mut applibs = match std::env::current_dir() {
        Ok(cdir) => get_libraries(cdir.join(PathBuf::from("data"))),
        Err(e) => {
            error!(
                "could not get application built-in libraries at working directory: {}",
                e
            );
            Vec::new()
        }
    };
    info!("added application libraries to catalogue");
    let mut doclibs = match document_dir() {
        Some(ddir) => {
            get_libraries(ddir.join(PathBuf::from("ProcFlow").join(PathBuf::from("Libraries"))))
        }
        _ => {
            error!("could not get libraries at document directory");
            Vec::new()
        }
    };
    info!("added document libraries to catalogue");

    libs.push(internal);
    libs.append(&mut applibs);
    libs.append(&mut doclibs);
    libs
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Catalogue {
    pub libraries: std::collections::HashMap<uuid::Uuid, Library>,
}

impl Catalogue {
    pub fn new() -> Catalogue {
        Catalogue {
            libraries: Default::default(),
        }
    }

    pub fn load_default_libraries(&mut self) {
        self.libraries.clear();
        for lib in load_libraries() {
            self.libraries.insert(lib.info.uuid, lib);
        }
    }

    pub fn get_graph_info(&self, id: uuid::Uuid) -> Option<GraphInfo> {
        for lib in self.libraries.values() {
            if let Some(graph) = lib.graphs.get(&id) {
                return Some(graph.info.clone());
            };
        }
        None
    }

    pub fn get_graph_ref(&self, id: uuid::Uuid, version: u64) -> Option<GraphRef> {
        for lib in self.libraries.values() {
            if let Some(graph) = lib.graphs.get(&id) {
                return Some(GraphRef {
                    name: graph.info.name.clone(),
                    uuid: graph.info.uuid,
                    library: Some(lib.info.uuid),
                    version,
                });
            };
        }
        None
    }

    pub fn get_graph_version(&self, graph_ref: &GraphRef) -> Option<VersionInfo> {
        for lib in self.libraries.values() {
            if let Some(graph) =
                get_graph_version_from_library(&lib, graph_ref.uuid, graph_ref.version)
            {
                return Some(graph.clone());
            };
        }
        None
    }

    pub fn has_graph_version(&self, graph_ref: &GraphRef) -> bool {
        for lib in self.libraries.values() {
            if has_graph_version_from_library(&lib, graph_ref.uuid, graph_ref.version) {
                return true;
            };
        }
        false
    }
}
