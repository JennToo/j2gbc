#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate log;

extern crate j2gbc;

use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

use gfx::traits::{Factory, FactoryExt};
use gfx::Device;
use glutin::{Event, GlContext, KeyboardInput, VirtualKeyCode, WindowEvent};

fn load_system(cart_path: &str) -> j2gbc::system::System {
    let cart_file = File::open(cart_path.clone()).unwrap();
    let mut c = j2gbc::cart::Cart::load(cart_file).unwrap();
    let save_path = format!("{}.sav", cart_path);
    if let Ok(mut f) = File::open(&save_path) {
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_ok() {
            println!("Loaded save file {}", save_path);
        }
        c.set_sram(buf.as_slice());
    }

    info!("Loaded cart {}:", cart_path);
    info!("Name: {}", c.name());
    info!("File Size: {} bytes", c.data.len());
    info!("Cart type: {}", c.type_());
    info!("ROM Size: {} bytes", c.rom_size());
    info!("RAM Size: {} bytes", c.ram_size());

    let sink = j2gbc::audio::NullSink;

    let cpu = j2gbc::cpu::Cpu::new(c, Box::new(sink));
    j2gbc::system::System::new(cpu)
}

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

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

pub fn main() {
    let mut args = std::env::args();
    let cart_path = args.nth(1).unwrap();
    let mut timer = DeltaTimer::new();
    let mut system = load_system(&cart_path);

    let mut events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title(format!("j2gbc -- {}", cart_path))
        .with_dimensions((1024, 768).into());

    let (api, version, vs_code, fs_code) = (
        glutin::Api::OpenGl,
        (3, 2),
        include_bytes!("../shader/lcd_vert.glsl").to_vec(),
        include_bytes!("../shader/lcd_frag.glsl").to_vec(),
    );

    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(api, version))
        .with_vsync(true);
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop);
    let mut encoder = gfx::Encoder::from(factory.create_command_buffer());

    let lcd_tex = factory
        .create_texture::<gfx::format::R8_G8_B8_A8>(
            gfx::texture::Kind::D2(160, 144, gfx::texture::AaMode::Single),
            1,
            gfx::memory::Bind::SHADER_RESOURCE,
            gfx::memory::Usage::Dynamic,
            Some(gfx::format::ChannelType::Unorm),
        ).unwrap();
    let lcd_view = factory
        .view_texture_as_shader_resource::<(gfx::format::R8_G8_B8_A8, gfx::format::Unorm)>(
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

    let pso = factory
        .create_pipeline_simple(&vs_code, &fs_code, pipe::new())
        .unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&QUAD, ());
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color,
        lcd: (lcd_view, lcd_sampler),
    };

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => running = false,
                    WindowEvent::Resized(size) => {
                        window.resize(size.to_physical(window.get_hidpi_factor()));
                        gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    }
                    _ => (),
                }
            }
        });

        let elapsed = timer.elapsed();
        if elapsed > Duration::from_millis(17) {
            info!(target: "events", "Slow frame {:?}", elapsed);
        }
        system.run_for_duration(&elapsed);

        encoder.clear(&data.out, CLEAR_COLOR);
        encoder
            .update_texture::<gfx::format::R8_G8_B8_A8, (gfx::format::R8_G8_B8_A8, gfx::format::Unorm)>(
                &lcd_tex,
                None,
                lcd_tex.get_info().to_image_info(0),
                system.get_framebuffer().raw(),
            ).unwrap();

        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

struct DeltaTimer {
    last_time: Instant,
}

impl DeltaTimer {
    fn new() -> DeltaTimer {
        DeltaTimer {
            last_time: Instant::now(),
        }
    }

    fn elapsed(&mut self) -> Duration {
        let new_now = Instant::now();
        let d = new_now - self.last_time;
        self.last_time = new_now;
        d
    }
}
