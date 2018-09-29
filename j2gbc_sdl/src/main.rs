#![allow(unknown_lints)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate cpal;
extern crate hound;
extern crate j2ds;
extern crate sdl2;

use std::fs::File;
use std::io::Read;

pub mod emu;
pub mod ui;

fn main() {
    ui::debug::install_logger();

    let mut args = std::env::args();
    let cart_path = args.nth(1).unwrap();
    let cart_file = File::open(cart_path.clone()).unwrap();
    let mut c = emu::cart::Cart::load(cart_file).unwrap();
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

    let sink = ui::audio::CpalSink::new().unwrap();

    let cpu = emu::cpu::Cpu::new(c, Box::new(sink));
    let system = emu::system::System::new(cpu);

    let mut window = ui::Window::new(save_path).unwrap();
    window.run(system).unwrap();
    info!("Exiting gracefully");
}
