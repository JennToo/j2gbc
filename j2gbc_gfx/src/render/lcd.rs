use gfx;
use gfx::traits::{Factory, FactoryExt};
use j2gbc::lcd::fb::SCREEN_SIZE;
use j2gbc::system::System;

use super::*;

pub struct LcdRender {
    lcd_tex: gfx::handle::Texture<ResourcesT, SurfaceFormat>,
    pso: gfx::PipelineState<ResourcesT, pipe::Meta>,
    data: pipe::Data<ResourcesT>,
    slice: gfx::Slice<ResourcesT>,
}

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

impl LcdRender {
    pub fn new(factory: &mut FactoryT, main_color: &ColorHandle) -> LcdRender {
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

        let (vs_code, fs_code) = (
            include_bytes!("../../shader/lcd_vert.glsl").to_vec(),
            include_bytes!("../../shader/lcd_frag.glsl").to_vec(),
        );
        let pso = factory
            .create_pipeline_simple(&vs_code, &fs_code, pipe::new())
            .unwrap();
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&QUAD, ());
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
            ).unwrap();

        encoder.draw(&self.slice, &self.pso, &self.data);
    }
}
