use serde::{Deserialize, Serialize};

use dirs::document_dir;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

use log::*;

use uuid::*;

use super::graph::*;
use super::library::*;

fn load_libraries() -> Vec<Library> {
    println!("test");
    let applibs = match std::env::current_dir() {
        Ok(cdir) => get_libraries(cdir.join(PathBuf::from("data"))),
        Err(e) => {
            error!(
                "could not get application built-in libraries at working directory: {}",
                e
            );
            Vec::new()
        }
    };
    let doclibs = match document_dir() {
        Some(ddir) => {
            get_libraries(ddir.join(PathBuf::from("ProcFlow").join(PathBuf::from("Libraries"))))
        }
        _ => {
            error!("could not get libraries at document directory");
            Vec::new()
        }
    };

    let libs = applibs
        .into_iter()
        .chain(doclibs.into_iter())
        .collect::<Vec<_>>();
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
                return Some(GraphRef {name: graph.info.name.clone(), uuid: graph.info.uuid, library: Some(lib.info.uuid), version});
            };
        }
        None
    }

    pub fn get_graph_version(&self, id: uuid::Uuid, version: u64) -> Option<VersionInfo> {
        for lib in self.libraries.values() {
            if let Some(graph) = get_graph_version_from_library(&lib, id, version) {
                return Some(graph.clone());
            };
        }
        None
    }
}
