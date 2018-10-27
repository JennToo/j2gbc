use gfx;
use gfx::Device;
use glutin::GlContext;
use j2gbc::system::System;
use std::time::Duration;

mod lcd;
mod ui;

pub type ResourcesT = gfx_device_gl::Resources;
pub type EncoderT = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;
pub type DeviceT = gfx_device_gl::Device;
pub type FactoryT = gfx_device_gl::Factory;
pub type Window = glutin::GlWindow;
pub type DepthHandle = gfx::handle::DepthStencilView<gfx_device_gl::Resources, DepthFormat>;
pub type ColorHandle = gfx::handle::RenderTargetView<gfx_device_gl::Resources, ColorFormat>;
pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type SurfaceFormat = gfx::format::R8_G8_B8_A8;

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

pub struct Renderer {
    encoder: EncoderT,
    device: DeviceT,
    window: Window,
    depth: DepthHandle,
    factory: FactoryT,
    main_color: ColorHandle,

    lcd: lcd::LcdRender,
    pub ui: ui::UiRender,
}

impl Renderer {
    pub fn new(window: glutin::GlWindow) -> Renderer {
        let (device, mut factory, mut main_color, depth) =
            gfx_window_glutin::init_existing::<ColorFormat, DepthFormat>(&window);
        let encoder = gfx::Encoder::from(factory.create_command_buffer());

        let lcd = lcd::LcdRender::new(&mut factory, &mut main_color);
        let ui = ui::UiRender::new(&device, &window, &mut factory, &mut main_color);

        Renderer {
            encoder,
            device,
            window,
            depth,
            factory,
            ui,
            lcd,
            main_color,
        }
    }

    pub fn draw(&mut self, system: &mut System, dt: Duration) {
        self.encoder.clear(&self.main_color, CLEAR_COLOR);

        self.lcd.draw(&mut self.encoder, system);
        self.ui
            .draw(dt, &mut self.encoder, &mut self.factory, system);

        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().unwrap();
        self.device.cleanup();
    }

    pub fn resize(&mut self, size: glutin::dpi::LogicalSize) {
        self.window
            .resize(size.to_physical(self.window.get_hidpi_factor()));
        gfx_window_glutin::update_views(&self.window, &mut self.main_color, &mut self.depth);
    }
}
