use serde::{Deserialize, Serialize};

use dirs::document_dir;
use std::io;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use log::*;

use uuid::*;

#[derive(Serialize, Deserialize)]
pub struct LibInfo {
    pub name: String,
    pub uuid: uuid::Uuid,
    pub author: String,
    pub format: u64
}
 
pub struct Library {
    pub info: LibInfo,
    pub path: PathBuf
}

pub fn get_libraries(libs: &mut Vec<Library>) -> io::Result<()> {
    match document_dir() {
        Some(dpath) => {
            info!("docs: {}", dpath.to_string_lossy());
            let libraries_path = dpath.join(Path::new("ProcFlow/Libraries"));
            info!("searching in {} for libraries", libraries_path.display());
            for entry in walkdir::WalkDir::new(&libraries_path) {
                let entry = entry?;
                if entry.path().is_dir() {
                    let libjson = entry.path().to_path_buf().join(Path::new("lib.json"));
                    info!("walking {}", entry.path().display());
                    if libjson.is_file() {
                        match std::fs::read_to_string(&libjson) {
                            Ok(json) => {
                                match serde_json::from_str(&json) {
                                    Ok(info) => {
                                        libs.push(Library {
                                            info: info,
                                            path: entry.path().to_path_buf()
                                        });
                                    },
                                    Err(e) => error!("could not parse {}: {}", libjson.display(), e)
                                }
                            },
                            Err(e) => error!("could not open {}: {}", libjson.display(), e)
                        }
                    }
                }
            }
        },
        None => warn!("could not find documents folder"),
    }
    Ok(())
}