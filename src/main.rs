extern crate proc_flow_lib as pf;

// use proc_flow_lib as pf;

#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

fn main() {
    let config = ConfigBuilder::new().add_filter_ignore_str("axiom").build();
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, config.clone(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            config.clone(),
            std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open("log.txt")
                .expect("could not open logging file at binary"),
        ),
    ])
    .unwrap();
    
    let mut engine = pf::engine::Engine::new();
    let canvas_node_id = uuid::Uuid::parse_str("a795c3e9-0a2f-48bf-a9e2-03378e8e59b8").unwrap();
    let canvas_node_edit_recv_id = uuid::Uuid::parse_str("7c5c2794-eb60-4661-9d25-585e1226233e").unwrap();
    let canvas_actor = engine.boot_graph(canvas_node_id, 1, uuid::Uuid::new_v4());
    match canvas_actor {
        Some(canvas_actor) => {
            engine.send_value(canvas_actor.clone(), canvas_node_edit_recv_id, None);
        }
        None => panic!("did not get aid back from engine boot of canvas node"),
    };

    info!("Hello, world!");
}
