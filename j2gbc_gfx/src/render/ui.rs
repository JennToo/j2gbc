use imgui::{FontGlyphRange, FrameSize, ImFontConfig, ImGui, ImVec4};
use imgui_gfx_renderer::{Renderer, Shaders};
use std::time::Duration;

use glutin::ElementState::Pressed;
use glutin::WindowEvent::*;
use glutin::{Event, MouseButton, MouseScrollDelta, TouchPhase};

use super::*;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub struct UiRender {
    renderer: Renderer<ResourcesT>,
    frame_size: FrameSize,
    ctx: ImGui,

    mouse_state: MouseState,
}

impl UiRender {
    pub fn new(
        device: &DeviceT,
        window: &Window,
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

        UiRender {
            renderer,
            frame_size,
            ctx: imgui,
            mouse_state: MouseState::default(),
        }
    }

    pub fn draw(&mut self, delta_time: Duration, encoder: &mut EncoderT, factory: &mut FactoryT) {
        let ui = self.ctx.frame(self.frame_size, delta_time.as_secs() as f32);

        let mut b = true;
        ui.show_demo_window(&mut b);

        self.renderer.render(ui, factory, encoder).unwrap();
    }

    // TODO: Break this out of the "renderer"
    pub fn handle_event(&mut self, event: &Event) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                CursorMoved { position: pos, .. } => {
                    // Rescale position from glutin logical coordinates to our logical
                    // coordinates
                    self.mouse_state.pos = pos
                        .to_physical(1.) // TODO: Put the HiDPI stuff back
                        .to_logical(1.)
                        .into();
                }
                MouseInput { state, button, .. } => match button {
                    MouseButton::Left => self.mouse_state.pressed.0 = *state == Pressed,
                    MouseButton::Right => self.mouse_state.pressed.1 = *state == Pressed,
                    MouseButton::Middle => self.mouse_state.pressed.2 = *state == Pressed,
                    _ => {}
                },
                MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_, y),
                    phase: TouchPhase::Moved,
                    ..
                } => self.mouse_state.wheel = *y,
                MouseWheel {
                    delta: MouseScrollDelta::PixelDelta(pos),
                    phase: TouchPhase::Moved,
                    ..
                } => {
                    // Rescale pixel delta from glutin logical coordinates to our logical
                    // coordinates
                    self.mouse_state.wheel = pos
                        .to_physical(1.)
                        .to_logical(1.)
                        .y as f32;
                }
                ReceivedCharacter(c) => self.ctx.add_input_character(*c),
                _ => (),
            }
        }
        // TODO: This is inefficient
        self.update_mouse();
    }

    fn update_mouse(&mut self) {
        self.ctx.set_mouse_pos(self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32);
        self.ctx.set_mouse_down([
            self.mouse_state.pressed.0,
            self.mouse_state.pressed.1,
            self.mouse_state.pressed.2,
            false,
            false,
        ]);
        self.ctx.set_mouse_wheel(self.mouse_state.wheel);
        self.mouse_state.wheel = 0.0;
    }
}
