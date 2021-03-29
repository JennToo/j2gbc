use crate::error::ExecutionError;
use crate::{cpu::Cpu, lcd::fb::Framebuffer, mem::MemDevice};
pub use crate::{cpu::Register8, inst::Instruction, lcd::BG_SIZE, mem::Address};

pub struct Debugger<'a> {
    cpu: &'a mut Cpu,
}

impl<'a> Debugger<'a> {
    pub fn new(cpu: &'a mut Cpu) -> Self {
        Debugger { cpu }
    }

    pub fn read_reg(&self, reg: Register8) -> u8 {
        self.cpu[reg]
    }

    pub fn read_sp(&self) -> Address {
        self.cpu.sp
    }

    pub fn read_pc(&self) -> Address {
        self.cpu.pc
    }

    pub fn read_ime(&self) -> bool {
        self.cpu.interrupt_master_enable
    }

    pub fn read_mem(&self, addr: Address) -> Result<u8, ExecutionError> {
        self.cpu.mmu.read(addr)
    }

    pub fn is_halted_on_debugger(&self) -> bool {
        self.cpu.debug_halted
    }

    pub fn resume(&mut self) {
        self.cpu.debug_halted = false;
    }

    pub fn pause(&mut self) {
        self.cpu.debug_halted = true;
    }

    pub fn step(&mut self) {
        #![allow(unused)]
        self.cpu.run_cycle();
    }

    pub fn fetch_instruction(&self, addr: Address) -> Result<(Instruction, u8), ExecutionError> {
        self.cpu.fetch_instruction(addr)
    }

    pub fn add_breakpoint(&mut self, addr: Address) {
        self.cpu.breakpoints.insert(addr);
    }

    pub fn remove_breakpoint(&mut self, addr: Address) {
        self.cpu.breakpoints.remove(&addr);
    }

    pub fn get_breakpoints(&self) -> impl Iterator<Item = &Address> {
        self.cpu.breakpoints.iter()
    }

    pub fn render_bg_to_fb(&self, index: usize, output: &mut Framebuffer) {
        self.cpu.mmu.lcd.render_bg_to_fb(index, output);
    }
}
