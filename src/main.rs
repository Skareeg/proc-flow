extern crate proc_flow_lib;

use proc_flow_lib as pf;

#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;
use axiom::prelude::*;

#[macro_use] extern crate conrod_core;

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
            WriteLogger::new(LevelFilter::Info, Config::default(), std::fs::OpenOptions::new().append(true).create(true).open("log.txt").expect("could not open logging file at binary")),
        ]
    ).unwrap();

    info!("Hello, world!");
    let mut catalogue = pf::catalogue::Catalogue::new();
    catalogue.load_default_libraries();

    match catalogue.get_graph_version(uuid::Uuid::parse_str("2361f8a5-2330-41e8-a4ad-492b3c15febe").unwrap(), 1) {
        Some(version) => {
            info!("found graphics node version");
        },
        None => {
            error!("could not find graphics node version");
        }
    }

    widget_ids! {
        pub struct Ids1 {
            button,
            pin1,
            pin2,
        }
    }
    widget_ids! {
        pub struct Ids2 {
            button,
            pin1,
            pin2,
        }
    }

    let mut tui = conrod_core::UiBuilder::new([640.0, 480.0])
        .build();
    let ids1 = Ids1::new(tui.widget_id_generator());
    let ids2 = Ids2::new(tui.widget_id_generator());

    info!("Ids1: {:?} {:?} {:?}", ids1.button, ids1.pin1, ids1.pin2);
    info!("Ids2: {:?} {:?} {:?}", ids2.button, ids2.pin1, ids2.pin2);
    info!("Anon: {:?} {:?} {:?}", tui.widget_id_generator().next(), tui.widget_id_generator().next(), tui.widget_id_generator().next());

    let mut system = ActorSystem::create(ActorSystemConfig::default());
}
