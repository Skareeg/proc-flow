mod library;

use library::*;
use log::*;

pub fn test() {
    println!("test");
    let mut libs = Vec::new();
    match get_libraries(&mut libs) {
        Err(e) => error!("could not get libraries: {}", e),
        _ => {}
    }
    for lib in libs {
        info!("library found and loaded: {}", lib.info.name);
    }
}