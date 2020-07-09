use iced_winit::winit;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// For the ability to launch on a separate thread.
use winit::platform::windows::EventLoopExtWindows;

use iced_native::{UserInterface, Cache, Size, Clipboard, MouseCursor};
use iced_wgpu::Renderer;
use iced_wgpu::wgpu;

use futures::executor::block_on;

pub fn run_canvas_editor() {
    let event_loop = EventLoop::<()>::new_any_thread();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut logical_size = window.inner_size().to_logical::<f64>(window.scale_factor());
    let mut modifiers = winit::event::ModifiersState::default();

    let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        backends: wgpu::BackendBit::PRIMARY,
    }).expect("failed to get the adapter");

    // Create the logical device and command queue
    let (mut device, mut queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions::default(),
                limits: wgpu::Limits::default(),
            },
        );

    let size = window.inner_size();

    let surface = wgpu::Surface::create(&window);
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut swap_chain = {
        iced_wgpu::window::SwapChain::new(&device, &surface, format, size.width, size.height)
    };
    let mut resized = false;

    let mut events = Vec::new();
    let mut cache = Some(Cache::new());
    let mut renderer = Renderer::new(&mut device, iced_wgpu::Settings::default());
    let mut output = (iced_wgpu::Primitive::None, MouseCursor::OutOfBounds);

    // Create GUI elements here.

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {event, ..} => {
                match event {
                    WindowEvent::ModifiersChanged(new_mods) => {
                        modifiers = new_mods;
                    }
                    WindowEvent::Resized(new_size) => {
                        logical_size = new_size.to_logical(window.scale_factor());
                        resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    }
                    _ => {}
                }

                if let Some(event) = iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers) {
                    events.push(event);
                }
            }
            Event::MainEventsCleared => {
                if events.is_empty() {
                    return;
                }

                // let mut ui = UserInterface::build(root: E, bounds: Size, cache: Cache, renderer: &mut Renderer)

                window.request_redraw();
            }
            Event::RedrawRequested(windowId) => {
                if resized {
                    let size = window.inner_size();

                    swap_chain = iced_wgpu::window::SwapChain::new(&device, &surface, format, size.width, size.height);
                }

                let (frame, viewport) = swap_chain.next_frame();

                let mut encoder = device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor { todo: 0 },
                );

                let mouse_cursor = renderer.draw(&mut device, &mut encoder, iced_wgpu::Target { texture: &frame.view, viewport }, &output, window.scale_factor(), &["Some debug information!"]);

                queue.submit(&[encoder.finish()]);

                window.set_cursor_icon(iced_winit::conversion::mouse_cursor(mouse_cursor));
            }
            _ => {}
        }
    });
}