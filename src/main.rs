extern crate proc_flow_lib;

use proc_flow_lib::test;

#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
            WriteLogger::new(LevelFilter::Info, Config::default(), std::fs::OpenOptions::new().append(true).create(true).open("log.txt").expect("could not open logging file at binary")),
        ]
    ).unwrap();

    info!("Hello, world!");
    test();

    // use vulkano::device::{Device, DeviceExtensions, Features};

    // let (device, mut queues) = {
    //     Device::new(physical, &Features::none(), &DeviceExtensions::none(), [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    // };

    // let queue = queues.next().unwrap();
}
