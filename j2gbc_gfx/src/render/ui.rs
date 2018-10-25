use std::time::Duration;

use glutin::Event;
use imgui::*;
use imgui_gfx_renderer::{Renderer, Shaders};
use imgui_glutin_support;
use j2gbc::cpu::Register8;

use super::*;

pub struct UiRender {
    renderer: Renderer<ResourcesT>,
    frame_size: FrameSize,
    ctx: ImGui,
}

impl UiRender {
    pub fn new(
        device: &DeviceT,
        window: &super::Window,
        factory: &mut FactoryT,
        main_color: &ColorHandle,
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
            include_bytes!("../../../j2gbc_sdl/MOZART_0.ttf"),
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

        imgui_glutin_support::configure_keys(&mut imgui);

        UiRender {
            renderer,
            frame_size,
            ctx: imgui,
        }
    }

    pub fn draw(
        &mut self,
        delta_time: Duration,
        encoder: &mut EncoderT,
        factory: &mut FactoryT,
        system: &System,
    ) {
        let ui = self.ctx.frame(self.frame_size, delta_time.as_secs() as f32);

        ui.window(im_str!("Registers"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!(
                    " A: 0x{:02x}   F: 0x{:02x}    SP: {}",
                    system.cpu[Register8::A],
                    system.cpu[Register8::F],
                    system.cpu.sp
                ));
                ui.text(im_str!(
                    " B: 0x{:02x}   C: 0x{:02x}    PC: {}",
                    system.cpu[Register8::B],
                    system.cpu[Register8::C],
                    system.cpu.pc
                ));
                ui.text(im_str!(
                    " D: 0x{:02x}   E: 0x{:02x}   IME: {}",
                    system.cpu[Register8::D],
                    system.cpu[Register8::E],
                    system.cpu.interrupt_master_enable
                ));
                ui.text(im_str!(
                    " H: 0x{:02x}   L: 0x{:02x}",
                    system.cpu[Register8::H],
                    system.cpu[Register8::L]
                ));
            });

        self.renderer.render(ui, factory, encoder).unwrap();
    }

    pub fn handle_event(&mut self, event: &Event) {
        imgui_glutin_support::handle_event(&mut self.ctx, &event);
    }
}
