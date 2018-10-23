use gfx;
use gfx::traits::{Factory, FactoryExt};
use gfx::Device;
use gfx_device_gl;
use gfx_window_glutin;
use glutin;
use glutin::GlContext;
use j2gbc::lcd::fb::SCREEN_SIZE;
use j2gbc::system::System;
use std::time::Duration;

use ui;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type SurfaceFormat = gfx::format::R8_G8_B8_A8;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
        lcd: gfx::TextureSampler<[f32; 4]> = "t_Lcd",
    }
}

const QUAD: [Vertex; 6] = [
    Vertex {
        pos: [-1., -1.],
        uv: [0., 1.],
    },
    Vertex {
        pos: [1., -1.],
        uv: [1., 1.],
    },
    Vertex {
        pos: [1., 1.],
        uv: [1., 0.],
    },
    Vertex {
        pos: [1., 1.],
        uv: [1., 0.],
    },
    Vertex {
        pos: [-1., 1.],
        uv: [0., 0.],
    },
    Vertex {
        pos: [-1., -1.],
        uv: [0., 1.],
    },
];

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

pub struct Renderer {
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    data: pipe::Data<gfx_device_gl::Resources>,
    device: gfx_device_gl::Device,
    window: glutin::GlWindow,
    pso: gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    depth: gfx::handle::DepthStencilView<gfx_device_gl::Resources, DepthFormat>,
    factory: gfx_device_gl::Factory,

    lcd_tex: gfx::handle::Texture<gfx_device_gl::Resources, SurfaceFormat>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
    ui: ui::UiRender,
}

impl Renderer {
    pub fn new(window: glutin::GlWindow) -> Renderer {
        let (vs_code, fs_code) = (
            include_bytes!("../shader/lcd_vert.glsl").to_vec(),
            include_bytes!("../shader/lcd_frag.glsl").to_vec(),
        );
        let (device, mut factory, mut main_color, depth) =
            gfx_window_glutin::init_existing::<ColorFormat, DepthFormat>(&window);
        let encoder = gfx::Encoder::from(factory.create_command_buffer());

        let lcd_tex = factory
            .create_texture::<SurfaceFormat>(
                gfx::texture::Kind::D2(
                    SCREEN_SIZE.0 as u16,
                    SCREEN_SIZE.1 as u16,
                    gfx::texture::AaMode::Single,
                ),
                1,
                gfx::memory::Bind::SHADER_RESOURCE,
                gfx::memory::Usage::Dynamic,
                Some(gfx::format::ChannelType::Unorm),
            ).unwrap();
        let lcd_view = factory
            .view_texture_as_shader_resource::<(SurfaceFormat, gfx::format::Unorm)>(
                &lcd_tex,
                (1, 1),
                gfx::format::Swizzle(
                    gfx::format::ChannelSource::X,
                    gfx::format::ChannelSource::Y,
                    gfx::format::ChannelSource::Z,
                    gfx::format::ChannelSource::W,
                ),
            ).unwrap();
        let lcd_sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Scale,
            gfx::texture::WrapMode::Clamp,
        ));

        let ui = ui::UiRender::new(&device, &window, &mut factory, &mut main_color);

        let pso = factory
            .create_pipeline_simple(&vs_code, &fs_code, pipe::new())
            .unwrap();
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&QUAD, ());
        let data = pipe::Data {
            vbuf: vertex_buffer,
            out: main_color,
            lcd: (lcd_view, lcd_sampler),
        };

        Renderer {
            encoder,
            data,
            lcd_tex,
            device,
            window,
            pso,
            slice,
            depth,
            factory,
            ui,
        }
    }

    pub fn draw(&mut self, system: &System, dt: Duration) {
        self.encoder.clear(&self.data.out, CLEAR_COLOR);
        self.encoder
            .update_texture::<SurfaceFormat, (SurfaceFormat, gfx::format::Unorm)>(
                &self.lcd_tex,
                None,
                self.lcd_tex.get_info().to_image_info(0),
                system.get_framebuffer().raw(),
            ).unwrap();

        self.encoder.draw(&self.slice, &self.pso, &self.data);

        self.ui.draw(dt, &mut self.encoder, &mut self.factory);

        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().unwrap();
        self.device.cleanup();
    }

    pub fn resize(&mut self, size: glutin::dpi::LogicalSize) {
        self.window
            .resize(size.to_physical(self.window.get_hidpi_factor()));
        gfx_window_glutin::update_views(&self.window, &mut self.data.out, &mut self.depth);
    }
}
