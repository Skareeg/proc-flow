use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice, QueueFamily};
use vulkano::device::{Device, DeviceExtensions, Features};
use std::sync::Arc;

struct Vulkan {
    instance: &'static Arc<Instance>,
    physical: PhysicalDevice<'static>,
    queue_family: Arc<QueueFamily<'static>>
}

impl Vulkan {
    pub fn init_default<'a>(&'a mut self) {
        self.instance = &'a Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");

        self.physical = PhysicalDevice::enumerate(&self.instance).next().expect("no device available");

        for family in self.physical.queue_families() {
            println!("found a family with {:?} queue(s)", family.queues_count());
        }

        self.queue_family = Arc::new(self.physical.queue_families().find(|&q| q.supports_graphics()).expect("couldn't find a graphical queue family"));
    }
}