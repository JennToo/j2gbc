#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate log;

extern crate j2gbc;

use std::fs::File;
use std::io::Read;
use std::time::Duration;

use glutin::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};

mod render;
mod timer;

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

pub fn main() {
    let mut args = std::env::args();
    let cart_path = args.nth(1).unwrap();
    let mut timer = timer::DeltaTimer::new();
    let mut system = load_system(&cart_path);

    let mut events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title(format!("j2gbc -- {}", cart_path))
        .with_dimensions((1024, 768).into());
    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window_config, context, &events_loop).unwrap();

    let mut renderer = render::Renderer::new(gl_window);

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
                        renderer.resize(size);
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

        renderer.draw(&system);
    }
}
