use proc_flow_lib as pf;

#[test]
fn calculates_output() {
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
                            assert_eq!("testing log actor", *value);
                        },
                        None => panic!("log actor test message was not a string")
                    }
                },
                None => panic!("retrieved empty log actor test message")
            }
        }
        None => panic!("did not get aid back from engine boot of log node"),
    };
    assert_eq!(pf::axiom::prelude::ShutdownResult::Ok, engine.wait(std::time::Duration::from_secs(5)));
}