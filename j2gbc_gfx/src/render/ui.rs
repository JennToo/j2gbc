use std::time::Duration;

use gfx::traits::Factory;
use glutin::Event;
use imgui::*;
use imgui_gfx_renderer::{Renderer, Shaders};
use imgui_glutin_support;
use j2gbc::{
    debug::{Address, Register8, BG_SIZE},
    Framebuffer,
};

use super::*;

const INSTRUCTION_PRINT_COUNT: usize = 40;

pub struct UiRender {
    renderer: Renderer<ResourcesT>,
    frame_size: FrameSize,
    ctx: ImGui,
    debugger_ui: DebuggerUi,
    logger_ui: LoggerUi,
    disassembly_ui: DisassemblyUi,
    breakpoints_ui: BreakpointsUi,
    visibility_set: VisibilitySet,

    bg_tex: gfx::handle::Texture<ResourcesT, SurfaceFormat>,
    bg_fb: Framebuffer,
    bg_im_tex: ImTexture,
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
        let font_size = (14.0 * hidpi_factor) as f32;

        imgui.fonts().add_default_font_with_config(
            ImFontConfig::new()
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size),
        );

        imgui.fonts().add_font_with_config(
            include_bytes!("../../../assets/fonts/DejaVuSansMono.ttf"),
            ImFontConfig::new()
                .merge_mode(true)
                .oversample_h(2)
                .pixel_snap_h(true)
                .size_pixels(font_size)
                .rasterizer_multiply(1.),
            &FontGlyphRange::japanese(),
        );

        imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

        let mut renderer =
            Renderer::init(&mut imgui, factory, shaders, main_color.clone()).unwrap();
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

        // TODO: We actually need 2 of these
        let bg_tex = factory
            .create_texture::<SurfaceFormat>(
                gfx::texture::Kind::D2(
                    BG_SIZE.0 as u16,
                    BG_SIZE.1 as u16,
                    gfx::texture::AaMode::Single,
                ),
                1,
                gfx::memory::Bind::SHADER_RESOURCE,
                gfx::memory::Usage::Dynamic,
                Some(gfx::format::ChannelType::Unorm),
            )
            .unwrap();
        let bg_sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Scale,
            gfx::texture::WrapMode::Clamp,
        ));
        let bg_view = factory
            .view_texture_as_shader_resource::<(SurfaceFormat, gfx::format::Unorm)>(
                &bg_tex,
                (1, 1),
                gfx::format::Swizzle(
                    gfx::format::ChannelSource::X,
                    gfx::format::ChannelSource::Y,
                    gfx::format::ChannelSource::Z,
                    gfx::format::ChannelSource::W,
                ),
            )
            .unwrap();
        let bg_im_tex = renderer.textures().insert((bg_view, bg_sampler));
        let bg_fb = Framebuffer::new(BG_SIZE);

        UiRender {
            renderer,
            frame_size,
            bg_tex,
            bg_im_tex,
            bg_fb,
            ctx: imgui,
            debugger_ui: DebuggerUi::default(),
            logger_ui: LoggerUi::default(),
            disassembly_ui: DisassemblyUi::default(),
            breakpoints_ui: BreakpointsUi::default(),
            visibility_set: VisibilitySet::default(),
        }
    }

    // It is unfortunate how the UI actions are so tightly coupled to the rendering of that UI,
    // but it seems to be a purposeful design decision in imgui, so there isn't much we can do
    // about it
    pub fn draw(
        &mut self,
        delta_time: Duration,
        encoder: &mut EncoderT,
        factory: &mut FactoryT,
        system: &mut System,
    ) {
        let time = delta_time.as_secs() as f32 + delta_time.subsec_nanos() as f32 / 1_000_000_000.;
        let mut ui = self.ctx.frame(self.frame_size, time);

        let visibility_set = &mut self.visibility_set;
        ui.main_menu_bar(|| {
            ui.menu(im_str!("View")).build(|| {
                let ret = ui.menu_item(im_str!("Debugger")).build();
                if ret {
                    visibility_set.debugger_ui = true;
                }
                let ret = ui.menu_item(im_str!("Logger")).build();
                if ret {
                    visibility_set.logger_ui = true;
                }
                let ret = ui.menu_item(im_str!("Disassembly")).build();
                if ret {
                    visibility_set.disassembly_ui = true;
                }
                let ret = ui.menu_item(im_str!("Breakpoints")).build();
                if ret {
                    visibility_set.breakpoints_ui = true;
                }
            });
        });

        self.debugger_ui
            .draw(&mut ui, &mut visibility_set.debugger_ui, system);
        self.logger_ui.draw(&mut ui, &mut visibility_set.logger_ui);
        self.disassembly_ui
            .draw(&mut ui, &mut visibility_set.disassembly_ui, system);
        self.breakpoints_ui
            .draw(&mut ui, &mut visibility_set.breakpoints_ui, system);

        // TODO: Refactor
        if visibility_set.background_ui {
            system.debugger().render_bg_to_fb(0, &mut self.bg_fb);
            encoder
                .update_texture::<SurfaceFormat, (SurfaceFormat, gfx::format::Unorm)>(
                    &self.bg_tex,
                    None,
                    self.bg_tex.get_info().to_image_info(0),
                    self.bg_fb.raw(),
                )
                .unwrap();
            let bg_im_tex = self.bg_im_tex.clone();
            ui.window(im_str!("BG"))
                .always_auto_resize(true)
                .opened(&mut visibility_set.background_ui)
                .collapsible(false)
                .build(|| {
                    Image::new(
                        &ui,
                        bg_im_tex,
                        ImVec2::new(BG_SIZE.0 as f32 * 2., BG_SIZE.1 as f32 * 2.),
                    )
                    .build();
                });
        }

        self.renderer.render(ui, factory, encoder).unwrap();
    }

    pub fn handle_event(&mut self, event: &Event) {
        imgui_glutin_support::handle_event(&mut self.ctx, &event);
    }

    pub fn update_render_target(&mut self, out: ColorHandle) {
        self.renderer.update_render_target(out);
    }

    pub fn resize(&mut self, size: glutin::dpi::LogicalSize, hidpi_factor: f64) {
        let physical_size = size.to_physical(hidpi_factor);
        let logical_size = physical_size.to_logical(hidpi_factor);

        self.frame_size = FrameSize {
            logical_size: logical_size.into(),
            hidpi_factor,
        };
    }
}

struct VisibilitySet {
    debugger_ui: bool,
    logger_ui: bool,
    disassembly_ui: bool,
    breakpoints_ui: bool,
    background_ui: bool,
}

impl Default for VisibilitySet {
    fn default() -> VisibilitySet {
        VisibilitySet {
            debugger_ui: true,
            logger_ui: true,
            disassembly_ui: true,
            breakpoints_ui: true,
            background_ui: true,
        }
    }
}

#[derive(Default)]
struct DebuggerUi {
    cache: Option<DebuggerCache>,
}

struct DebuggerCache {
    registers: ImString,
}

impl DebuggerUi {
    fn draw<'a, 'ui>(&mut self, ui: &mut Ui<'ui>, visibility: &'a mut bool, system: &mut System) {
        if !*visibility {
            return;
        }
        ui.window(im_str!("Debugger"))
            .always_auto_resize(true)
            .opened(visibility)
            .collapsible(false)
            .build(|| {
                if !system.debugger().is_halted_on_debugger() {
                    if ui.button(im_str!("Pause"), ImVec2::new(100., 25.)) {
                        system.debugger().pause();
                        self.cache = None;
                    }
                } else {
                    if ui.button(im_str!("Resume"), ImVec2::zero()) {
                        system.debugger().resume();
                        self.cache = None;
                    }
                    ui.same_line(0.);
                    if ui.button(im_str!("Step"), ImVec2::zero()) {
                        system.debugger().step();
                        self.cache = None;
                    }

                    ui.separator();

                    if self.cache.is_none() {
                        self.cache = Some(Self::generate_cache(system));
                    }

                    let cache = self.cache.as_ref().unwrap();

                    ui.text(&cache.registers);
                }
            });
    }

    fn generate_cache(system: &mut System) -> DebuggerCache {
        let debug = system.debugger();
        let registers = im_str!(
            " A: 0x{:02x}   F: 0x{:02x}    SP: {}
 B: 0x{:02x}   C: 0x{:02x}    PC: {}
 D: 0x{:02x}   E: 0x{:02x}   IME: {}
 H: 0x{:02x}   L: 0x{:02x}",
            debug.read_reg(Register8::A),
            debug.read_reg(Register8::F),
            debug.read_sp(),
            debug.read_reg(Register8::B),
            debug.read_reg(Register8::C),
            debug.read_pc(),
            debug.read_reg(Register8::D),
            debug.read_reg(Register8::E),
            debug.read_ime(),
            debug.read_reg(Register8::H),
            debug.read_reg(Register8::L),
        )
        .clone();

        DebuggerCache { registers }
    }
}

#[derive(Default)]
struct LoggerUi;

impl LoggerUi {
    fn draw<'a, 'ui>(&mut self, ui: &mut Ui<'ui>, visibility: &'a mut bool) {
        if !*visibility {
            return;
        }
        ui.window(im_str!("Logger"))
            .size((600., 300.), ImGuiCond::FirstUseEver)
            .opened(visibility)
            .build(|| {
                let mut records = crate::logger::DEBUG_LOGGER.log.lock().unwrap();

                if ui.button(im_str!("Clear"), ImVec2::zero()) {
                    records.clear();
                }

                for record in &*records {
                    ui.text(im_str!(
                        "{}:{:03}",
                        record.timestamp.as_secs(),
                        record.timestamp.subsec_millis(),
                    ));
                    ui.same_line(0.);
                    ui.text_wrapped(ImString::new(record.message.clone()).as_ref());
                    ui.separator();
                }
            });
    }
}

#[derive(Default)]
struct DisassemblyUi {
    cache: Option<DisassemblyCache>,
}

struct DisassemblyCache {
    start_address: Address,
    disassembly: ImString,
}

impl DisassemblyUi {
    fn draw<'a, 'ui>(&mut self, ui: &mut Ui<'ui>, visibility: &'a mut bool, system: &mut System) {
        if !*visibility {
            return;
        }
        ui.window(im_str!("Disassembly"))
            .size((600., 300.), ImGuiCond::FirstUseEver)
            .opened(visibility)
            .build(|| {
                if !system.debugger().is_halted_on_debugger() {
                    ui.text(im_str!("Pause execution to view disassembly"));
                    return;
                }
                if self.cache.is_none()
                    || self.cache.as_ref().unwrap().start_address != system.debugger().read_pc()
                {
                    self.generate_cache(system);
                }
                let cache = self.cache.as_ref().unwrap();
                ui.text(&cache.disassembly);
            });
    }

    fn generate_cache(&mut self, system: &mut System) {
        let debug = system.debugger();
        let mut disassembly = String::default();

        let mut address = debug.read_pc();
        for _ in 0..INSTRUCTION_PRINT_COUNT {
            match debug.fetch_instruction(address) {
                Result::Ok((ins, len)) => {
                    if address == debug.read_pc() {
                        disassembly += format!(" => {}: {}\n", address, ins).as_str();
                    } else {
                        disassembly += format!("    {}: {}\n", address, ins).as_str();
                    }
                    address += Address(u16::from(len));
                }
                Result::Err(()) => {
                    disassembly += format!("{}: Invalid\n", address).as_str();
                    address += Address(1);
                }
            }
        }
        self.cache = Some(DisassemblyCache {
            start_address: debug.read_pc(),
            disassembly: ImString::new(disassembly),
        });
    }
}

struct BreakpointsUi {
    address_buffer: ImString,
}

impl BreakpointsUi {
    fn default() -> BreakpointsUi {
        BreakpointsUi {
            address_buffer: ImString::with_capacity(32),
        }
    }
}

impl BreakpointsUi {
    fn draw<'a, 'ui>(&mut self, ui: &mut Ui<'ui>, visibility: &'a mut bool, system: &mut System) {
        if !*visibility {
            return;
        }

        let mut debug = system.debugger();

        ui.window(im_str!("Breakpoints"))
            .always_auto_resize(true)
            .opened(visibility)
            .collapsible(false)
            .build(|| {
                if ui
                    .input_text(im_str!(""), &mut self.address_buffer)
                    .chars_hexadecimal(true)
                    .enter_returns_true(true)
                    .build()
                {
                    let address =
                        Address(u16::from_str_radix(self.address_buffer.to_str(), 16).unwrap());
                    debug.add_breakpoint(address);
                    self.address_buffer = ImString::with_capacity(32);
                }

                ui.separator();

                let mut to_remove = vec![];
                for bp in debug.get_breakpoints() {
                    if ui.button(im_str!("X"), ImVec2::zero()) {
                        to_remove.push(*bp);
                    }
                    ui.same_line(0.);
                    ui.text(im_str!("{}", bp));
                }
                for r in &to_remove {
                    debug.remove_breakpoint(*r);
                }
            });
    }
}
