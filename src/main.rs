extern crate rust_flow_lib;

use rust_flow_lib::test;

fn main() {
    println!("Hello, world!");
    test();

    // use vulkano::device::{Device, DeviceExtensions, Features};

    // let (device, mut queues) = {
    //     Device::new(physical, &Features::none(), &DeviceExtensions::none(), [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    // };

    // let queue = queues.next().unwrap();
}
