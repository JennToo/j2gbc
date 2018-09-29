use std::time::Duration;

use super::cpu::Cpu;
use super::lcd::Framebuffer;

pub struct System {
    pub cpu: Cpu,
}

impl System {
    pub fn new(cpu: Cpu) -> System {
        System { cpu }
    }

    pub fn run_for_duration(&mut self, duration: &Duration) {
        self.cpu.run_for_duration(duration);
    }

    pub fn get_framebuffer(&self) -> &Framebuffer {
        self.cpu.mmu.lcd.get_framebuffer()
    }
}
