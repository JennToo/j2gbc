use std::time::Duration;

use super::cpu::Cpu;

pub struct System {
    cpu: Cpu,
}

impl System {
    pub fn new(cpu: Cpu) -> System {
        System { cpu }
    }

    pub fn run_for_duration(&mut self, duration: &Duration) {
        self.cpu.run_for_duration(duration);
    }
}
