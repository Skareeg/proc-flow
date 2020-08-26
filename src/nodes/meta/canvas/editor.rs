use iced_winit::winit;
use iced_wgpu::wgpu;

// For the ability to launch on a separate thread.
use winit::platform::windows::EventLoopExtWindows;

// For the ability to return from the event loop on Exit.
use winit::platform::desktop::EventLoopExtDesktop;

use futures::executor::block_on;
use crossbeam::{Receiver, Sender};

use super::canvas::CanvasMessage;

use axiom::prelude as ax;

use log::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ]
        }
    }
}

pub fn run_canvas_editor(node: ax::Aid, recv_from_editor_actor: Receiver<CanvasMessage>) {
    let mut event_loop = winit::event_loop::EventLoop::<()>::new_any_thread();
    let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
    let mut logical_size = window.inner_size().to_logical::<f64>(window.scale_factor());
    let mut cursor_position = winit::dpi::PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = winit::event::ModifiersState::default();

    let (surface, mut device, queue) = create_device(&window);
    let win_size = window.inner_size();
    let mut swap_chain = create_standard_swap_chain(win_size.width, win_size.height, &surface, &mut device);

    let mut viewport = iced_wgpu::Viewport::with_physical_size(
        iced_winit::Size::new(win_size.width, win_size.height),
        window.scale_factor(),
    );

    let mut resized = false;

    let scene = Scene::new(&mut device);
    let user_interface = UserInterface {};

    // Initialize iced
    let mut debug = iced_winit::Debug::new();
    let mut renderer = iced_wgpu::Renderer::new(iced_wgpu::Backend::new(&mut device, iced_wgpu::Settings::default()));

    let mut state = iced_winit::program::State::new(
        user_interface,
        viewport.logical_size(),
        iced_winit::conversion::cursor_position(cursor_position, viewport.scale_factor()),
        &mut renderer,
        &mut debug,
    );

    // Create GUI elements here.

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent {event, ..} => {
                match event {
                    winit::event::WindowEvent::ModifiersChanged(new_mods) => {
                        modifiers = new_mods;
                    }
                    winit::event::WindowEvent::Resized(new_size) => {
                        logical_size = new_size.to_logical(window.scale_factor());
                        resized = true;
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    _ => {}
                }

                if let Some(event) = iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers) {
                    // events.push(event);
                }
            }
            winit::event::Event::MainEventsCleared => {
                // if events.is_empty() {
                //     return;
                // }

                // let mut ui = UserInterface::build(root: E, bounds: Size, cache: Cache, renderer: &mut Renderer)

                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(windowId) => {
                if resized {
                    let size = window.inner_size();
                    swap_chain = create_standard_swap_chain(size.width, size.height, &surface, &mut device);
                    resized = false;
                }

                let frame = swap_chain.get_next_texture().expect("Next frame");

                let mut encoder = device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor { label: None },
                );

                // let program = state.program();

                {
                    // We clear the frame
                    let mut render_pass = scene.clear(
                        &frame.view,
                        &mut encoder,
                        iced_winit::Color::BLACK
                    );

                    // Draw the scene
                    // TODO Different scene here
                    scene.draw(&mut render_pass);
                }

                // And then iced on top
                let mouse_interaction = renderer.backend_mut().draw(
                    &mut device,
                    &mut encoder,
                    &frame.view,
                    &viewport,
                    state.primitive(),
                    &debug.overlay(),
                );

                // Then we submit the work
                queue.submit(&[encoder.finish()]);

                // And update the mouse cursor
                window.set_cursor_icon(
                    iced_winit::conversion::mouse_interaction(
                        mouse_interaction,
                    ),
                );
            }
            _ => {}
        }
    });
    
    let _ = node.send_new(CanvasMessage::Exit);
}

// Code taken from https://github.com/hecrj/iced/blob/master/examples/integration/src/scene.rs
// Credits to Iced team: https://github.com/hecrj/iced
// License MIT at https://github.com/hecrj/iced/blob/master/LICENSE

pub struct Scene {
    pipeline: wgpu::RenderPipeline,
}

impl Scene {
    pub fn new(device: &wgpu::Device) -> Scene {
        let pipeline = build_standard_pipeline(device);

        Scene { pipeline }
    }

    pub fn clear<'a>(
        &self,
        target: &'a wgpu::TextureView,
        encoder: &'a mut wgpu::CommandEncoder,
        background_color: iced_winit::Color,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: target,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: {
                    let [r, g, b, a] = background_color.into_linear();

                    wgpu::Color {
                        r: r as f64,
                        g: g as f64,
                        b: b as f64,
                        a: a as f64,
                    }
                },
            }],
            depth_stencil_attachment: None,
        })
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..3, 0..1);
    }
}

struct UserInterface {
}

#[derive(Debug, Clone)]
enum UserInterfaceMessage {
}

impl iced_native::program::Program for UserInterface {
    type Renderer = iced_wgpu::Renderer;
    type Message = UserInterfaceMessage;

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&mut self) -> iced_winit::Element<Self::Message, Self::Renderer> {
        iced::Row::new()
            .into()
    }
}

pub fn build_standard_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
    let vs = include_bytes!("shader/vert.spv");
    let fs = include_bytes!("shader/frag.spv");

    let vs_module = device.create_shader_module(
        &wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap(),
    );

    let fs_module = device.create_shader_module(
        &wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap(),
    );

    let pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[],
        });

    let pipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    Vertex::desc(),
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

    pipeline
}

pub fn create_device(window: &winit::window::Window) -> (wgpu::Surface, wgpu::Device, wgpu::Queue) {
    // Initialize wgpu
    let surface = wgpu::Surface::create(window);
    let (mut device, queue) = futures::executor::block_on(async {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .expect("Request adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await
    });
    (
        surface
        ,device
        ,queue
    )
}

pub fn create_standard_swap_chain(width: u32, height: u32, surface: &wgpu::Surface, device: &mut wgpu::Device) -> wgpu::SwapChain {
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Mailbox,
        },
    )
}