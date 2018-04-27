use std::ops::{Index, IndexMut};
use std::num::Wrapping;
use std::time::Duration;
use std::collections::{HashSet, VecDeque};
use std::cmp::min;

use super::alu::{add, and, dec, hi, hi_lo, inc, lo, or, sub, xor, Flags, add16};
use super::inst::{Arith, Control, Instruction, Load, Logic};
use super::mem::{Address, MemDevice};
use super::mmu::Mmu;
use super::debug::debug;
use super::cart::Cart;

pub const CLOCK_RATE: u64 = 4_190_000;

mod register;
mod interrupt;

pub use self::register::{Register16, Register8};
pub use self::interrupt::Interrupt;

pub struct Cpu {
    registers: [u8; 8],
    pub pc: Address,
    pub sp: Address,
    pub mmu: Mmu,
    cycle: u64,
    pub interrupt_master_enable: bool,
    halted: bool,

    pub last_instructions: VecDeque<(Address, Instruction)>,
    pub breakpoints: HashSet<Address>,
}

impl Cpu {
    pub fn new(c: Cart) -> Cpu {
        let mut initial_breakpoints = HashSet::new();
        initial_breakpoints.insert(Address(0x0100));
        Cpu {
            registers: [0, 0, 0, 0, 0, 0, 0, 0],
            sp: Address(0xFFFE),
            pc: Address(0x100),
            mmu: Mmu::new(c),
            cycle: 0,
            interrupt_master_enable: false,
            halted: false,

            last_instructions: VecDeque::new(),
            breakpoints: initial_breakpoints,
        }
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    fn execute(&mut self, i: Instruction) -> Result<(), ()> {
        match i {
            Instruction::Nop => {}
            Instruction::Ei => {
                self.interrupt_master_enable = true;
            }
            Instruction::Di => {
                self.interrupt_master_enable = false;
            }
            Instruction::Halt => {
                println!("Halting");
                self.halted = true;
            }
            Instruction::Res(b, r) => {
                let v = self[r] & !(1 << b);
                self[r] = v;
            }
            Instruction::CpI(v) => {
                let (_, flags) = sub(self[Register8::A], v);
                self[Register8::F] = flags.0;
            }
            Instruction::CpR(r) => {
                let v = self[r];
                let (_, flags) = sub(self[Register8::A], v);
                self[Register8::F] = flags.0;
            }
            Instruction::Arith(a) => {
                try!(self.execute_arith(a));
            }
            Instruction::Control(c) => {
                try!(self.execute_control(c));
            }
            Instruction::Load(l) => {
                try!(self.execute_load(l));
            }
            Instruction::Logic(l) => {
                try!(self.execute_logic(l));
            }
        }
        self.cycle += i.cycles() as u64;
        Ok(())
    }

    fn execute_arith(&mut self, a: Arith) -> Result<(), ()> {
        match a {
            Arith::AddN => {
                let v1 = try!(self.read_indirect(Register16::HL));
                let v2 = self[Register8::A];
                let (v, flags) = add(v1, v2);
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Arith::AddR(r) => {
                let v1 = self[r];
                let v2 = self[Register8::A];
                let (v, flags) = add(v1, v2);
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Arith::IncR(r) => {
                let (v, flags) = inc(self[r], self.flags());
                self[r] = v;
                self[Register8::F] = flags.0;
            }
            Arith::IncN => {
                let s = try!(self.read_indirect(Register16::HL));
                let (v, flags) = inc(s, self.flags());
                try!(self.write_indirect(Register16::HL, v));
                self[Register8::F] = flags.0;
            }
            Arith::DecR(r) => {
                let (v, flags) = dec(self[r], self.flags());
                self[r] = v;
                self[Register8::F] = flags.0;
            }
            Arith::DecR16(r) => {
                let v = Wrapping(self.read_r16(r));
                self.write_r16(r, (v - Wrapping(1)).0);
            }
            Arith::IncR16(r) => {
                let v = Wrapping(self.read_r16(r));
                self.write_r16(r, (v + Wrapping(1)).0);
            }
            Arith::AddRR16(d, s) => {
                let v1 = self.read_r16(d);
                let v2 = self.read_r16(s);
                let (v3, flags) = add16(v1, v2, self.flags());
                self.write_r16(d, v3);
                self[Register8::F] = flags.0;
            }
        }
        Ok(())
    }

    fn execute_control(&mut self, c: Control) -> Result<(), ()> {
        match c {
            Control::JrNZI(o) => {
                if !self.flags().get_zero() {
                    self.pc += o;
                }
            }
            Control::JrNCI(o) => {
                if !self.flags().get_carry() {
                    self.pc += o;
                }
            }
            Control::JrI(o) => {
                self.pc += o;
            }
            Control::JrZI(o) => {
                if self.flags().get_zero() {
                    self.pc += o;
                }
            }
            Control::JrCI(o) => {
                if self.flags().get_carry() {
                    self.pc += o;
                }
            }
            Control::Ret => {
                self.pc = Address(try!(self.mmu.read16(self.sp)));
                self.sp += Address(2);
            }
            Control::Reti => {
                self.pc = Address(try!(self.mmu.read16(self.sp)));
                self.sp += Address(2);
                self.interrupt_master_enable = true;
            }
            Control::JpI(a) => {
                self.pc = a;
            }
            Control::JpCI(a) => {
                if self.flags().get_carry() {
                    self.pc = a;
                }
            }
            Control::JpZI(a) => {
                if self.flags().get_zero() {
                    self.pc = a;
                }
            }
            Control::JpNCI(a) => {
                if !self.flags().get_carry() {
                    self.pc = a;
                }
            }
            Control::JpNZI(a) => {
                if !self.flags().get_zero() {
                    self.pc = a;
                }
            }
            Control::CallI(a) => {
                let v = self.pc.into();
                try!(self.push16(v));
                self.pc = a;
            }
        }

        Ok(())
    }

    fn execute_load(&mut self, l: Load) -> Result<(), ()> {
        match l {
            Load::LdRM(r, a) => {
                self[r] = try!(self.mmu.read(a));
            }
            Load::LdMR(a, r) => {
                let v = self[r];
                try!(self.mmu.write(a, v));
            }
            Load::LdRR(d, s) => {
                let v = self[s];
                self[d] = v;
            }
            Load::LdRI(r, i) => {
                self[r] = i;
            }
            Load::LdNA(d) => {
                let a = self.read_r16(Register16::HL);
                let v = self[Register8::A];
                try!(self.mmu.write(Address(a), v));
                self.write_r16(Register16::HL, (Wrapping(a) + Wrapping(d as u16)).0);
            }
            Load::LdAN(d) => {
                let a = self.read_r16(Register16::HL);
                self[Register8::A] = try!(self.mmu.read(Address(a)));
                self.write_r16(Register16::HL, (Wrapping(a) + Wrapping(d as u16)).0);
            }
            Load::LdRN(r) => {
                let v = try!(self.read_indirect(Register16::HL));
                self[r] = v;
            }
            Load::LdNCA => {
                let a = Address(self[Register8::C] as u16 + 0xFF00);
                let v = self[Register8::A];
                try!(self.mmu.write(a, v));
            }
            Load::LdANC => {
                let a = Address(self[Register8::C] as u16 + 0xFF00);
                let v = try!(self.mmu.read(a));
                self[Register8::A] = v;
            }
            Load::LdNI(v) => {
                try!(self.write_indirect(Register16::HL, v));
            }
            Load::LdNR16(r) => {
                let v = self[Register8::A];
                try!(self.write_indirect(r, v))
            }
            Load::LdRN16(r) => {
                let v = try!(self.read_indirect(r));
                self[Register8::A] = v;
            }
            Load::LdRI16(r, i) => {
                self.write_r16(r, i);
            }
            Load::LdNIA16(a) => {
                let v = self[Register8::A];
                try!(self.mmu.write(a, v));
            }
            Load::LdANI16(a) => {
                let v = try!(self.mmu.read(a));
                self[Register8::A] = v;
            }
            Load::Pop(r) => {
                let v = try!(self.pop16());
                self.write_r16(r, v);
            }
            Load::Push(r) => {
                let v = self.read_r16(r);
                try!(self.push16(v));
            }
        }

        Ok(())
    }

    fn execute_logic(&mut self, l: Logic) -> Result<(), ()> {
        match l {
            Logic::AndI(v) => {
                let (value, flags) = and(self[Register8::A], v);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
            Logic::AndR(r) => {
                let (value, flags) = and(self[Register8::A], self[r]);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
            Logic::AndN => {
                let v = try!(self.read_indirect(Register16::HL));
                let (value, flags) = and(self[Register8::A], v);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
            Logic::OrI(v) => {
                let (value, flags) = or(self[Register8::A], v);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
            Logic::OrR(r) => {
                let (value, flags) = or(self[Register8::A], self[r]);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
            Logic::OrN => {
                let v = try!(self.read_indirect(Register16::HL));
                let (value, flags) = or(self[Register8::A], v);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
            Logic::XorR(r) => {
                let (value, flags) = xor(self[Register8::A], self[r]);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
        }

        Ok(())
    }

    pub fn run_cycle(&mut self) -> Result<(), ()> {
        if self.halted {
            return Ok(());
        }

        if self.breakpoints.contains(&self.pc) {
            self.breakpoints.remove(&self.pc);
            println!("Breakpoint");
            return Err(());
        }

        let (instruction, len) = try!(self.fetch_instruction());
        if self.last_instructions.len() > 50 {
            self.last_instructions.pop_front();
        }
        self.last_instructions.push_back((self.pc, instruction));

        self.pc += Address(len as u16);
        try!(self.execute(instruction));

        self.drive_peripherals()
    }

    pub fn run_for_duration(&mut self, duration: &Duration) {
        let cycles_to_run = duration_to_cycle_count(&duration);
        let stop_at_cycle = self.cycle() + cycles_to_run;
        while self.cycle() < stop_at_cycle {
            if self.run_cycle().is_err() {
                debug(self);
            }

            if self.halted {
                self.cycle = min(self.mmu.lcd.get_next_event_cycle(), stop_at_cycle);
                if self.drive_peripherals().is_err() {
                    debug(self);
                }
            }
        }
    }

    fn drive_peripherals(&mut self) -> Result<(), ()> {
        if let Some(i) = self.mmu.lcd.pump_cycle(self.cycle) {
            try!(self.handle_interrupt(i));
        }
        Ok(())
    }

    fn handle_interrupt(&mut self, int: Interrupt) -> Result<(), ()> {
        if self.interrupt_master_enable && int.is_enabled(self.mmu.interrupt_enable) {
            let v = self.pc.into();
            try!(self.push16(v));

            self.pc = int.table_address();
            self.interrupt_master_enable = false;
        }

        if self.halted {
            self.halted = false;
        }

        Ok(())
    }

    pub fn fetch_instruction(&self) -> Result<(Instruction, u8), ()> {
        let bytes = [
            try!(self.mmu.read(self.pc)),
            try!(self.mmu.read(self.pc + Address(1))),
            try!(self.mmu.read(self.pc + Address(2))),
        ];
        Instruction::decode(bytes)
    }

    fn write_r16(&mut self, r: Register16, v: u16) {
        match r {
            Register16::SP => self.sp = Address(v),
            Register16::PC => self.pc = Address(v),
            Register16::AF => {
                self[Register8::A] = hi(v);
                self[Register8::F] = lo(v);
            }
            Register16::BC => {
                self[Register8::B] = hi(v);
                self[Register8::C] = lo(v);
            }
            Register16::DE => {
                self[Register8::D] = hi(v);
                self[Register8::E] = lo(v);
            }
            Register16::HL => {
                self[Register8::H] = hi(v);
                self[Register8::L] = lo(v);
            }
        }
    }

    fn read_r16(&self, r: Register16) -> u16 {
        match r {
            Register16::SP => self.sp.0,
            Register16::PC => self.pc.0,
            Register16::AF => hi_lo(self[Register8::A], self[Register8::F]),
            Register16::BC => hi_lo(self[Register8::B], self[Register8::C]),
            Register16::DE => hi_lo(self[Register8::D], self[Register8::E]),
            Register16::HL => hi_lo(self[Register8::H], self[Register8::L]),
        }
    }

    fn push16(&mut self, v: u16) -> Result<(), ()> {
        let nsp = self.sp - Address(2);
        try!(self.mmu.write16(nsp, v));
        self.sp = nsp;
        Ok(())
    }

    fn pop16(&mut self) -> Result<u16, ()> {
        let v = try!(self.mmu.read16(self.sp));
        self.sp += Address(2);
        Ok(v)
    }

    fn read_indirect(&self, r: Register16) -> Result<u8, ()> {
        let a = Address(self.read_r16(r));
        self.mmu.read(a)
    }

    fn write_indirect(&mut self, r: Register16, v: u8) -> Result<(), ()> {
        let a = Address(self.read_r16(r));
        self.mmu.write(a, v)
    }

    fn flags(&self) -> Flags {
        Flags(self[Register8::F])
    }
}

pub fn duration_to_cycle_count(duration: &Duration) -> u64 {
    // Clock for the CPU is 4.19 MHz
    const NSEC_PER_SEC: u64 = 1_000_000_000;
    let scount = duration.as_secs() * CLOCK_RATE;
    let ncount = (CLOCK_RATE * duration.subsec_nanos() as u64) / NSEC_PER_SEC;
    scount + ncount
}

impl Index<Register8> for Cpu {
    type Output = u8;

    fn index(&self, r: Register8) -> &u8 {
        &self.registers[r as usize]
    }
}

impl IndexMut<Register8> for Cpu {
    fn index_mut(&mut self, r: Register8) -> &mut u8 {
        &mut self.registers[r as usize]
    }
}
