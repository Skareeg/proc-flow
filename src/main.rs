extern crate proc_flow_lib;

use proc_flow_lib as pf;

#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Clone)]
struct Test {
    pub msg: Option<serde_json::Value>,
}

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

    info!("Hello, world!");

    let mut engine = pf::engine::Engine::new();
    let log_id = uuid::Uuid::parse_str("fd41d8ef-d10f-4499-8a90-35b73d8ff246").unwrap();
    let log_info_input_pin_id = uuid::Uuid::parse_str("5e6ab872-5cca-4e01-8dbb-2df843102dc0").unwrap();
    let log_info_output_pin_id = uuid::Uuid::parse_str("44a986b1-dc09-45d9-ab65-e2c0c7b6f5ce").unwrap();
    let log_actor = engine.boot_graph(log_id, 1, uuid::Uuid::new_v4());
    match log_actor {
        Some(log_actor) => {
            engine.set_input_pin_value(log_actor.clone(), log_info_input_pin_id, Some(axiom::prelude::Message::new("testing log actor".to_owned())), "string".to_owned());
            let value = engine.get_output_pin_value(log_actor.clone(), log_info_output_pin_id, None);
            match value {
                Some(value) => {
                    match value.content_as::<String>() {
                        Some(value) => {
                            info!("retrieved node value of {:?}", value);
                        },
                        None => error!("log actor test message was not a string")
                    }
                },
                None => error!("retrieved empty log actor test message")
            }
            let value = engine.get_output_pin_value(log_actor.clone(), log_info_output_pin_id, None);
            match value {
                Some(value) => {
                    match value.content_as::<String>() {
                        Some(value) => {
                            info!("again retrieved node value of {:?}", value);
                        },
                        None => error!("log actor test message was not a string")
                    }
                },
                None => error!("retrieved empty log actor test message")
            }
        }
        None => error!("did not get aid back from engine boot of log node"),
    };
    engine.wait();
}
