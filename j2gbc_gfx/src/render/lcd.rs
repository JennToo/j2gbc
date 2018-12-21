use gfx::traits::{Factory, FactoryExt};
use gfx::{
    gfx_defines, gfx_impl_struct_meta, gfx_pipeline, gfx_pipeline_inner, gfx_vertex_struct_meta,
};
use j2gbc::{System, SCREEN_SIZE};

use super::*;

pub struct LcdRender {
    lcd_tex: gfx::handle::Texture<ResourcesT, SurfaceFormat>,
    pso: gfx::PipelineState<ResourcesT, pipe::Meta>,
    data: pipe::Data<ResourcesT>,
    slice: gfx::Slice<ResourcesT>,
}

gfx_defines! {
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

impl LcdRender {
    pub fn new(window: &Window, factory: &mut FactoryT, main_color: &ColorHandle) -> LcdRender {
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
            )
            .unwrap();
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
            )
            .unwrap();
        let lcd_sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Scale,
            gfx::texture::WrapMode::Clamp,
        ));

        let (vs_code, fs_code) = (
            include_bytes!("../../shader/lcd_vert.glsl").to_vec(),
            include_bytes!("../../shader/lcd_frag.glsl").to_vec(),
        );
        let pso = factory
            .create_pipeline_simple(&vs_code, &fs_code, pipe::new())
            .unwrap();
        let (vertex_buffer, slice) = upload_quad_for(factory, window.get_inner_size().unwrap());
        let data = pipe::Data {
            vbuf: vertex_buffer,
            out: main_color.clone(),
            lcd: (lcd_view, lcd_sampler),
        };

        LcdRender {
            data,
            lcd_tex,
            pso,
            slice,
        }
    }

    pub fn draw(&mut self, encoder: &mut EncoderT, system: &System) {
        encoder
            .update_texture::<SurfaceFormat, (SurfaceFormat, gfx::format::Unorm)>(
                &self.lcd_tex,
                None,
                self.lcd_tex.get_info().to_image_info(0),
                system.get_framebuffer().raw(),
            )
            .unwrap();

        encoder.draw(&self.slice, &self.pso, &self.data);
    }

    pub fn update_render_target(&mut self, out: ColorHandle) {
        self.data.out = out;
    }

    pub fn resize(&mut self, size: glutin::dpi::LogicalSize, factory: &mut FactoryT) {
        let (vertex_buffer, slice) = upload_quad_for(factory, size);
        self.slice = slice;
        self.data.vbuf = vertex_buffer;
    }
}

fn upload_quad_for(
    factory: &mut FactoryT,
    size: glutin::dpi::LogicalSize,
) -> (
    gfx::handle::Buffer<ResourcesT, Vertex>,
    gfx::Slice<ResourcesT>,
) {
    let window_size = (size.width as u32, size.height as u32);
    let (px, py) = proportional_subset((SCREEN_SIZE.0 as u32, SCREEN_SIZE.1 as u32), window_size);

    factory.create_vertex_buffer_with_slice(&make_centered_prop_quad(px, py), ())
}

fn make_centered_prop_quad(x_factor: f32, y_factor: f32) -> [Vertex; 6] {
    [
        Vertex {
            pos: [-1. * x_factor, -1. * y_factor],
            uv: [0., 1.],
        },
        Vertex {
            pos: [1. * x_factor, -1. * y_factor],
            uv: [1., 1.],
        },
        Vertex {
            pos: [1. * x_factor, 1. * y_factor],
            uv: [1., 0.],
        },
        Vertex {
            pos: [1. * x_factor, 1. * y_factor],
            uv: [1., 0.],
        },
        Vertex {
            pos: [-1. * x_factor, 1. * y_factor],
            uv: [0., 0.],
        },
        Vertex {
            pos: [-1. * x_factor, -1. * y_factor],
            uv: [0., 1.],
        },
    ]
}

fn proportional_subset(proportions: (u32, u32), max_size: (u32, u32)) -> (f32, f32) {
    let y_prop_corrected = max_size.0 * proportions.1 / proportions.0;
    let x_prop_corrected = max_size.1 * proportions.0 / proportions.1;
    if x_prop_corrected < y_prop_corrected {
        (x_prop_corrected as f32 / max_size.0 as f32, 1.)
    } else {
        (1., y_prop_corrected as f32 / max_size.1 as f32)
    }
}
