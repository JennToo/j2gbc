use gfx;
use gfx_device_gl;
use glutin;
use imgui::{FontGlyphRange, FrameSize, ImFontConfig, ImGui, ImVec4, ImGuiCond};
use imgui_gfx_renderer::{Renderer, Shaders};
use std::time::Duration;

use render::ColorFormat;

pub struct UiRender {
    renderer: Renderer<gfx_device_gl::Resources>,
    frame_size: FrameSize,
    ctx: ImGui,
}

impl UiRender {
    pub fn new(
        device: &gfx_device_gl::Device,
        window: &glutin::GlWindow,
        factory: &mut gfx_device_gl::Factory,
        main_color: &gfx::handle::RenderTargetView<gfx_device_gl::Resources, ColorFormat>,
    ) -> UiRender {
        let shaders = {
            let version = device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                Shaders::GlSl130
            } else {
                Shaders::GlSl110
            }
        };

        let mut imgui = ImGui::init();
        {
            // Fix incorrect colors with sRGB framebuffer
            fn imgui_gamma_to_linear(col: ImVec4) -> ImVec4 {
                let x = col.x.powf(2.2);
                let y = col.y.powf(2.2);
                let z = col.z.powf(2.2);
                let w = 1.0 - (1.0 - col.w).powf(2.2);
                ImVec4::new(x, y, z, w)
            }

            let style = imgui.style_mut();
            for col in 0..style.colors.len() {
                style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
            }
        }
        imgui.set_ini_filename(None);

        // In the examples we only use integer DPI factors, because the UI can get very blurry
        // otherwise. This might or might not be what you want in a real application.
        let hidpi_factor = window.get_hidpi_factor().round();

        let font_size = (13.0 * hidpi_factor) as f32;

        imgui.fonts().add_default_font_with_config(
            ImFontConfig::new()
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size),
        );

        imgui.fonts().add_font_with_config(
            include_bytes!("../../j2gbc_sdl/MOZART_0.ttf"),
            ImFontConfig::new()
                .merge_mode(true)
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size)
                .rasterizer_multiply(1.75),
            &FontGlyphRange::japanese(),
        );

        imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

        let renderer = Renderer::init(&mut imgui, factory, shaders, main_color.clone()).unwrap();
        let physical_size = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor());
        let logical_size = physical_size.to_logical(hidpi_factor);

        let frame_size = FrameSize {
            logical_size: logical_size.into(),
            hidpi_factor,
        };

        UiRender {
            renderer,
            frame_size,
            ctx: imgui,
        }
    }

    pub fn draw(
        &mut self,
        delta_time: Duration,
        encoder: &mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
        factory: &mut gfx_device_gl::Factory,
    ) {
        let ui = self.ctx.frame(self.frame_size, delta_time.as_secs() as f32);

        ui.window(im_str!("Test Window"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("Hello world!"));
            });

        self.renderer.render(ui, factory, encoder).unwrap();
    }
}
