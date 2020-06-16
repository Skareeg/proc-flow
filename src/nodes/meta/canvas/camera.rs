use amethyst::ecs::prelude::*;
use amethyst::renderer::debug_drawing::*;
use amethyst::core::*;
use amethyst::window::*;

pub struct CanvasCamera {
    pub position: math::Vector2<f64>,
}

pub struct CanvasCameraGridLineSystem;

use std::sync::*;

impl<'s> System<'s> for CanvasCameraGridLineSystem {
    type SystemData = (WriteExpect<'s, Arc<Mutex<conrod_core::Ui>>>, ReadExpect<'s, CanvasCamera>, ReadExpect<'s, ScreenDimensions>, WriteExpect<'s, DebugLines>);
    fn run(&mut self, (ui, camera, dimensions, lines): Self::SystemData) {
        let mut ui = ui.lock();
    }
}