use std::io::Read;
use std::time::Duration;

use log::info;

use crate::{
    audio::AudioSink, cart::Cart, cpu::Cpu, debug::Debugger, input::Button, lcd::fb::Framebuffer,
};

pub struct System {
    cpu: Cpu,
}

impl System {
    pub fn new<R: Read>(
        cart_data: R,
        audio_sink: Box<dyn AudioSink>,
        allow_cgb_mode: bool,
    ) -> std::io::Result<System> {
        let c = Cart::load(cart_data)?;

        info!("Name: {}", c.name());
        info!("File Size: {} bytes", c.data.len());
        info!("Cart type: {}", c.type_());
        info!("ROM Size: {} bytes", c.rom_size());
        info!("RAM Size: {} bytes", c.ram_size());

        let cpu = Cpu::new(c, audio_sink, allow_cgb_mode);

        Ok(System { cpu })
    }

    pub fn run_for_duration(&mut self, duration: &Duration) {
        self.cpu.run_for_duration(duration);
    }

    pub fn get_framebuffer(&self) -> &Framebuffer {
        self.cpu.mmu.lcd.get_framebuffer()
    }

    pub fn set_mmu_pedantic(&mut self, pedantic: bool) {
        self.cpu.mmu.pedantic = pedantic;
    }

    pub fn load_cart_sram(&mut self, sram: &[u8]) {
        self.cpu.mmu.cart.set_sram(sram);
    }

    pub fn read_cart_sram(&self) -> &[u8] {
        self.cpu.mmu.cart.get_sram()
    }

    pub fn debugger(&mut self) -> Debugger {
        Debugger::new(&mut self.cpu)
    }

    pub fn activate_button(&mut self, button: Button) {
        self.cpu.mmu.input.activate_button(button);
        self.cpu.request_p1_int();
    }

    pub fn deactivate_button(&mut self, button: Button) {
        self.cpu.mmu.input.deactivate_button(button);
    }
}

#[derive(Copy, Clone)]
pub enum SystemMode {
    DMG,
    CGB,
}
