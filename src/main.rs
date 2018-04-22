extern crate linenoise;
extern crate sdl2;

use std::fs::File;

pub mod emu;
pub mod ui;

fn main() {
    let mut window = ui::Window::new().unwrap();

    let mut args = std::env::args();
    let cart_path = args.nth(1).unwrap();
    let cart_file = File::open(cart_path.clone()).unwrap();
    let c = emu::cart::Cart::load(cart_file).unwrap();

    println!("Loaded cart {}:", cart_path);
    println!("Name: {}", c.name());
    println!("File Size: {} bytes", c.data.len());
    println!("Cart type: {}", c.type_());
    println!("ROM Size: {} bytes", c.rom_size());
    println!("RAM Size: {} bytes", c.ram_size());

    window.run(emu::cpu::Cpu::new(c)).unwrap();
}
