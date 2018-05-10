#![allow(unknown_lints)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate sdl2;

use std::fs::File;

pub mod emu;
pub mod ui;

fn main() {
    ui::debug::install_logger();
    let mut window = ui::Window::new().unwrap();

    let mut args = std::env::args();
    let cart_path = args.nth(1).unwrap();
    let cart_file = File::open(cart_path.clone()).unwrap();
    let c = emu::cart::Cart::load(cart_file).unwrap();

    info!("Loaded cart {}:", cart_path);
    info!("Name: {}", c.name());
    info!("File Size: {} bytes", c.data.len());
    info!("Cart type: {}", c.type_());
    info!("ROM Size: {} bytes", c.rom_size());
    info!("RAM Size: {} bytes", c.ram_size());

    let cpu = emu::cpu::Cpu::new(c);
    let system = emu::system::System::new(cpu);
    window.run(system).unwrap();
}
