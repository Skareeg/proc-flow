///
/// The greator majority of this file was copy pasted from: https://github.com/PistonDevelopers/conrod/blob/master/backends/conrod_wgpu/examples/all_winit_wgpu.rs
/// The theme was copied from the referenced: https://github.com/PistonDevelopers/conrod/blob/master/backends/conrod_example_shared/src/lib.rs
/// 

use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
};

use axiom::prelude::*;

use crate::graph;

conrod_winit::v021_conversion_fns!();

const LOGO_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
const MSAA_SAMPLES: u32 = 4;

///
/// Holds a basic implementation of a graph editor.
/// 
pub struct GraphCanvas {
    pub events: EventLoop<()>,
    pub window: winit::window::Window,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub format: wgpu::TextureFormat,
    pub swap_chain_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,

    pub renderer: conrod_wgpu::Renderer,
    pub multisampbuffer: wgpu::TextureView,
    pub ui: conrod_core::Ui,

    pub graph: Option<graph::GraphRef>,
}

impl GraphCanvas {
    pub fn new() -> GraphCanvas {
        let event_loop = EventLoop::new();

        // Create the window and surface.
        #[cfg(not(feature = "gl"))]
        let (window, mut size, surface) = {
            let window = winit::window::WindowBuilder::new()
                .with_title("Conrod with wgpu")
                .with_inner_size(winit::dpi::LogicalSize {
                    width: 640,
                    height: 480,
                })
                .build(&event_loop)
                .unwrap();
            let size = window.inner_size();
            let surface = wgpu::Surface::create(&window);
            (window, size, surface)
        };
    
        // Select an adapter and gpu device.
        let adapter_opts = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            backends: wgpu::BackendBit::PRIMARY,
        };
        let adapter = wgpu::Adapter::request(&adapter_opts).unwrap();
        let extensions = wgpu::Extensions {
            anisotropic_filtering: false,
        };
        let limits = wgpu::Limits::default();
        let device_desc = wgpu::DeviceDescriptor { extensions, limits };
        let (device, mut queue) = adapter.request_device(&device_desc);
    
        // Create the swapchain.
        let format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let mut swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Vsync,
        };
        let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
    
        // Create the renderer for rendering conrod primitives.
        let mut renderer = conrod_wgpu::Renderer::new(&device, MSAA_SAMPLES, format);
    
        // The intermediary multisampled texture that will be resolved (MSAA).
        let mut multisampled_framebuffer =
            create_multisampled_framebuffer(&device, &swap_chain_desc, MSAA_SAMPLES);
    
        // Create Ui and Ids of widgets to instantiate
        let mut ui = conrod_core::UiBuilder::new([640 as f64, 480 as f64])
            .theme(theme())
            .build();

        GraphCanvas {
            events: event_loop,
            window: window,
            size: size,
            surface: surface,
            adapter: adapter,
            device: device,
            queue: queue,
            format: format,
            swap_chain_desc: swap_chain_desc,
            swap_chain: swap_chain,
            renderer: renderer,
            multisampbuffer: multisampled_framebuffer,
            ui: ui,
            graph: Default::default(),
        }

        // let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());
    
        // Load font from file
        // let assets = find_folder::Search::KidsThenParents(3, 5)
        //     .for_folder("assets")
        //     .unwrap();
        // let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        // ui.fonts.insert_from_file(font_path).unwrap();
    
        // // Load the Rust logo from our assets folder to use as an example image.
        // let logo_path = assets.join("images/rust.png");
        // let rgba_logo_image = image::open(logo_path)
        //     .expect("Couldn't load logo")
        //     .to_rgba();
    
        // // Create the GPU texture and upload the image data.
        // let (logo_w, logo_h) = rgba_logo_image.dimensions();
        // let logo_tex = create_logo_texture(&device, &mut queue, rgba_logo_image);
        // let logo = conrod_wgpu::Image {
        //     texture: logo_tex,
        //     texture_format: LOGO_TEXTURE_FORMAT,
        //     width: logo_w,
        //     height: logo_h,
        // };
        // let mut image_map = conrod_core::image::Map::new();
        // let rust_logo = image_map.insert(logo);
    }

    pub fn render(&mut self) {
    }

    pub async fn handle(mut self, ctx: Context, msg: &Message) -> Status {
        Status::Done
    }
}

/// A set of reasonable stylistic defaults that works for the `gui` below.
pub fn theme() -> conrod_core::Theme {
    use conrod_core::position::{Align, Direction, Padding, Position, Relative};
    conrod_core::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod_core::color::DARK_CHARCOAL,
        shape_color: conrod_core::color::LIGHT_CHARCOAL,
        border_color: conrod_core::color::BLACK,
        border_width: 0.0,
        label_color: conrod_core::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod_core::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    sample_count: u32,
) -> wgpu::TextureView {
    let multisampled_texture_extent = wgpu::Extent3d {
        width: sc_desc.width,
        height: sc_desc.height,
        depth: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        size: multisampled_texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: sc_desc.format,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    device
        .create_texture(multisampled_frame_descriptor)
        .create_default_view()
}

fn create_logo_texture(
    device: &wgpu::Device,
    queue: &mut wgpu::Queue,
    image: image::RgbaImage,
) -> wgpu::Texture {
    // Initialise the texture.
    let (width, height) = image.dimensions();
    let logo_tex_extent = wgpu::Extent3d {
        width,
        height,
        depth: 1,
    };
    let logo_tex = device.create_texture(&wgpu::TextureDescriptor {
        size: logo_tex_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: LOGO_TEXTURE_FORMAT,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });

    // Upload the pixel data.
    let data = &image.into_raw()[..];
    let buffer = device
        .create_buffer_mapped(data.len(), wgpu::BufferUsage::COPY_SRC)
        .fill_from_slice(data);

    // Submit command for copying pixel data to the texture.
    let pixel_size_bytes = 4; // Rgba8, as above.
    let buffer_copy_view = wgpu::BufferCopyView {
        buffer: &buffer,
        offset: 0,
        row_pitch: width * pixel_size_bytes,
        image_height: height,
    };
    let texture_copy_view = wgpu::TextureCopyView {
        texture: &logo_tex,
        mip_level: 0,
        array_layer: 0,
        origin: wgpu::Origin3d::ZERO,
    };
    let extent = wgpu::Extent3d {
        width: width,
        height: height,
        depth: 1,
    };
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor { todo: 0 };
    let mut encoder = device.create_command_encoder(&cmd_encoder_desc);
    encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);
    queue.submit(&[encoder.finish()]);

    logo_tex
}