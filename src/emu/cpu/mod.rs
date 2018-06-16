use std::cmp::min;
use std::collections::{HashSet, VecDeque};
use std::num::Wrapping;
use std::ops::{Index, IndexMut};
use std::time::Duration;

use super::alu::*;
use super::cart::Cart;
use super::inst::{Arith, Bits, Control, Instruction, Load, Logic};
use super::mem::{Address, ExtendedAddress, MemDevice};
use super::mmu::Mmu;

pub const CLOCK_RATE: u64 = 4_190_000;
pub const LONGEST_INSTRUCTION_CYCLE: u64 = 20; // LD (a16),SP

mod interrupt;
mod register;
#[cfg(test)]
mod test;

pub use self::interrupt::Interrupt;
pub use self::register::{Operand, Register16, Register8};

pub struct Cpu {
    registers: [u8; 8],
    pub pc: Address,
    pub sp: Address,
    pub mmu: Mmu,
    cycle: u64,
    pub interrupt_master_enable: bool,
    halted: bool,

    pub debug_halted: bool,
    pub last_instructions: VecDeque<(ExtendedAddress, Instruction)>,
    pub breakpoints: HashSet<Address>,
    pub interrupt_breakpoints: HashSet<Interrupt>,
}

impl Cpu {
    pub fn new(c: Cart) -> Cpu {
        let initial_breakpoints = HashSet::new();

        let mut cpu = Cpu {
            registers: [0, 0, 0, 0, 0, 0, 0, 0],
            sp: Address(0xFFFE),
            pc: Address(0x100),
            mmu: Mmu::new(c),
            cycle: 0,
            interrupt_master_enable: false,
            halted: false,

            debug_halted: false,
            last_instructions: VecDeque::new(),
            breakpoints: initial_breakpoints,
            interrupt_breakpoints: HashSet::new(),
        };

        cpu[Register8::A] = 0x01;
        cpu[Register8::F] = 0xB0;
        cpu[Register8::B] = 0x00;
        cpu[Register8::C] = 0x13;
        cpu[Register8::D] = 0x00;
        cpu[Register8::E] = 0xD8;
        cpu[Register8::H] = 0x01;
        cpu[Register8::L] = 0x4D;

        cpu
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
                self.halted = true;
            }
            Instruction::Scf => {
                let mut f = self.flags();
                f.set_subtract(false);
                f.set_halfcarry(false);
                f.set_carry(true);
                self[Register8::F] = f.0;
            }
            Instruction::Ccf => {
                let mut f = self.flags();
                f.set_subtract(false);
                f.set_halfcarry(false);
                f.set_carry(!self.flags().get_carry());
                self[Register8::F] = f.0;
            }
            Instruction::Cp(o) => {
                let v = self.read_operand(o)?;
                let (_, flags) = sub(self[Register8::A], v);
                self[Register8::F] = flags.0;
            }
            Instruction::Arith(a) => {
                self.execute_arith(a)?;
            }
            Instruction::Bits(b) => {
                self.execute_bits(b)?;
            }
            Instruction::Control(c) => {
                self.execute_control(c)?;
            }
            Instruction::Load(l) => {
                self.execute_load(l)?;
            }
            Instruction::Logic(l) => {
                self.execute_logic(l)?;
            }
        }
        self.cycle += u64::from(i.cycles());
        Ok(())
    }

    fn execute_arith(&mut self, a: Arith) -> Result<(), ()> {
        match a {
            Arith::Add(o) => {
                let v1 = self[Register8::A];
                let v2 = self.read_operand(o)?;
                let (v, flags) = add(v1, v2);
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Arith::Adc(o) => {
                let v1 = self[Register8::A];
                let v2 = self.read_operand(o)?;
                let (v, flags) = adc(v1, v2, self.flags().get_carry());
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Arith::Sub(o) => {
                let v1 = self[Register8::A];
                let v2 = self.read_operand(o)?;
                let (v, flags) = sub(v1, v2);
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Arith::Sbc(o) => {
                let v1 = self[Register8::A];
                let v2 = self.read_operand(o)?;
                let (v, flags) = sbc(v1, v2, self.flags().get_carry());
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Arith::Inc(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = inc(v, self.flags());
                self.write_operand(o, v)?;
                self[Register8::F] = flags.0;
            }
            Arith::Dec(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = dec(v, self.flags());
                self.write_operand(o, v)?;
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
            Arith::AddSP(v) => {
                let v1 = self.read_r16(Register16::SP);
                let v2 = (v as i16) as u16;
                let (v3, _) = add16(v1, v2, self.flags());
                // Flags are based on 8-bit addition
                let (_, mut flags) = add(v1 as u8, v as u8);
                flags.set_zero(false);
                flags.set_subtract(false);
                self.write_r16(Register16::SP, v3);
                self[Register8::F] = flags.0;
            }
            Arith::Daa => {
                let (v, f) = daa(self[Register8::A], self.flags());
                self[Register8::A] = v;
                self[Register8::F] = f.0;
            }
        }
        Ok(())
    }

    fn execute_bits(&mut self, b: Bits) -> Result<(), ()> {
        match b {
            Bits::Cpl => {
                let mut f = self.flags();
                f.set_subtract(true);
                f.set_halfcarry(true);
                let v = self[Register8::A];
                self[Register8::A] = !v;
                self[Register8::F] = f.0;
            }
            Bits::Swap(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = swap(v);
                self.write_operand(o, v)?;
                self[Register8::F] = flags.0;
            }
            Bits::Sla(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = sla(v);
                self.write_operand(o, v)?;
                self[Register8::F] = flags.0;
            }
            Bits::Sra(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = sra(v);
                self.write_operand(o, v)?;
                self[Register8::F] = flags.0;
            }
            Bits::Srl(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = srl(v);
                self.write_operand(o, v)?;
                self[Register8::F] = flags.0;
            }
            Bits::Rlc(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = rlc(v, self.flags());
                self.write_operand(o, v)?;
                self[Register8::F] = flags.0;
            }
            Bits::Rrc(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = rrc(v, self.flags());
                self.write_operand(o, v)?;
                self[Register8::F] = flags.0;
            }
            Bits::Rl(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = rl(v, self.flags());
                self.write_operand(o, v)?;
                self[Register8::F] = flags.0;
            }
            Bits::Rr(o) => {
                let v = self.read_operand(o)?;
                let (v, flags) = rr(v, self.flags());
                self.write_operand(o, v)?;
                self[Register8::F] = flags.0;
            }
            Bits::Rra => {
                let (v, mut flags) = rr(self[Register8::A], self.flags());
                flags.set_zero(false);
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Bits::Rla => {
                let (v, mut flags) = rl(self[Register8::A], self.flags());
                flags.set_zero(false);
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Bits::Rrca => {
                let (v, mut flags) = rrc(self[Register8::A], self.flags());
                flags.set_zero(false);
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Bits::Rlca => {
                let (v, mut flags) = rlc(self[Register8::A], self.flags());
                flags.set_zero(false);
                self[Register8::A] = v;
                self[Register8::F] = flags.0;
            }
            Bits::Bit(b, o) => {
                let v = self.read_operand(o)?;
                let mut flags = self.flags();
                flags.set_halfcarry(true);
                flags.set_subtract(false);
                flags.set_zero(v & (1 << b) == 0);
                self[Register8::F] = flags.0;
            }
            Bits::Res(b, o) => {
                let v = self.read_operand(o)?;
                self.write_operand(o, v & !(1 << b))?;
            }
            Bits::Set(b, o) => {
                let v = self.read_operand(o)?;
                self.write_operand(o, v | (1 << b))?;
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
                self.pc = Address(self.mmu.read16(self.sp)?);
                self.sp += Address(2);
            }
            Control::Reti => {
                self.pc = Address(self.mmu.read16(self.sp)?);
                self.sp += Address(2);
                self.interrupt_master_enable = true;
            }
            Control::RetC => {
                if self.flags().get_carry() {
                    self.pc = Address(self.mmu.read16(self.sp)?);
                    self.sp += Address(2);
                }
            }
            Control::RetZ => {
                if self.flags().get_zero() {
                    self.pc = Address(self.mmu.read16(self.sp)?);
                    self.sp += Address(2);
                }
            }
            Control::RetNC => {
                if !self.flags().get_carry() {
                    self.pc = Address(self.mmu.read16(self.sp)?);
                    self.sp += Address(2);
                }
            }
            Control::RetNZ => {
                if !self.flags().get_zero() {
                    self.pc = Address(self.mmu.read16(self.sp)?);
                    self.sp += Address(2);
                }
            }
            Control::JpN => {
                let a = Address(self.read_r16(Register16::HL));
                self.pc = a;
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
            Control::CallI(a) | Control::Rst(a) => {
                let v = self.pc.into();
                self.push16(v)?;
                self.pc = a;
            }
            Control::CallINZ(a) => {
                if !self.flags().get_zero() {
                    let v = self.pc.into();
                    self.push16(v)?;
                    self.pc = a;
                }
            }
            Control::CallINC(a) => {
                if !self.flags().get_carry() {
                    let v = self.pc.into();
                    self.push16(v)?;
                    self.pc = a;
                }
            }
            Control::CallIZ(a) => {
                if self.flags().get_zero() {
                    let v = self.pc.into();
                    self.push16(v)?;
                    self.pc = a;
                }
            }
            Control::CallIC(a) => {
                if self.flags().get_carry() {
                    let v = self.pc.into();
                    self.push16(v)?;
                    self.pc = a;
                }
            }
        }

        Ok(())
    }

    fn execute_load(&mut self, l: Load) -> Result<(), ()> {
        match l {
            Load::Ld(o1, o2) => {
                let v = self.read_operand(o2)?;
                self.write_operand(o1, v)?;
            }
            Load::LdNA(d) => {
                let a = self.read_r16(Register16::HL);
                let v = self[Register8::A];
                self.mmu.write(Address(a), v)?;
                self.write_r16(Register16::HL, (Wrapping(a) + Wrapping(d as u16)).0);
            }
            Load::LdAN(d) => {
                let a = self.read_r16(Register16::HL);
                self[Register8::A] = self.mmu.read(Address(a))?;
                self.write_r16(Register16::HL, (Wrapping(a) + Wrapping(d as u16)).0);
            }
            Load::LdNCA => {
                let a = Address(u16::from(self[Register8::C]) + 0xFF00);
                let v = self[Register8::A];
                self.mmu.write(a, v)?;
            }
            Load::LdANC => {
                let a = Address(u16::from(self[Register8::C]) + 0xFF00);
                let v = self.mmu.read(a)?;
                self[Register8::A] = v;
            }
            Load::LdHLSPI(v) => {
                let v1 = self.read_r16(Register16::SP);
                // Flags are based on 8-bit addition
                let (_, mut flags) = add(v1 as u8, v as u8);
                flags.set_zero(false);
                flags.set_subtract(false);

                let (v, _) = add16(v1, (v as i16) as u16, self.flags());
                flags.set_zero(false);
                flags.set_subtract(false);
                self.write_r16(Register16::HL, v);
                self[Register8::F] = flags.0;
            }
            Load::LdSPHL => {
                let v = self.read_r16(Register16::HL);
                self.write_r16(Register16::SP, v);
            }
            Load::LdIndirectSP(a) => {
                let v = self.read_r16(Register16::SP);
                self.mmu.write16(a, v)?;
            }
            Load::LdNR16(r) => {
                let v = self[Register8::A];
                self.write_indirect(r, v)?;
            }
            Load::LdRN16(r) => {
                let v = self.read_indirect(r)?;
                self[Register8::A] = v;
            }
            Load::LdRI16(r, i) => {
                self.write_r16(r, i);
            }
            Load::LdNIA16(a) => {
                let v = self[Register8::A];
                self.mmu.write(a, v)?;
            }
            Load::LdANI16(a) => {
                let v = self.mmu.read(a)?;
                self[Register8::A] = v;
            }
            Load::Pop(r) => {
                let v = self.pop16()?;
                self.write_r16(r, v);
            }
            Load::Push(r) => {
                let v = self.read_r16(r);
                self.push16(v)?;
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
                let v = self.read_indirect(Register16::HL)?;
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
                let v = self.read_indirect(Register16::HL)?;
                let (value, flags) = or(self[Register8::A], v);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
            Logic::XorI(v) => {
                let (value, flags) = xor(self[Register8::A], v);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
            Logic::XorR(r) => {
                let (value, flags) = xor(self[Register8::A], self[r]);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
            Logic::XorN => {
                let v = self.read_indirect(Register16::HL)?;
                let (value, flags) = xor(self[Register8::A], v);
                self[Register8::A] = value;
                self[Register8::F] = flags.0;
            }
        }

        Ok(())
    }

    fn read_operand(&self, o: Operand) -> Result<u8, ()> {
        match o {
            Operand::Immediate(v) => Ok(v),
            Operand::Register(r) => Ok(self[r]),
            Operand::IndirectRegister(ir) => self.read_indirect(ir),
            Operand::IndirectAddress(a) => self.mmu.read(a),
        }
    }

    fn write_operand(&mut self, o: Operand, v: u8) -> Result<(), ()> {
        match o {
            Operand::Immediate(_) => panic!("Invalid instruction requesting write to immediate"),
            Operand::Register(r) => {
                self[r] = v;
                Ok(())
            }
            Operand::IndirectAddress(a) => self.mmu.write(a, v),
            Operand::IndirectRegister(r) => self.write_indirect(r, v),
        }
    }

    pub fn run_cycle(&mut self) -> Result<(), ()> {
        self.fire_interrupts()?;

        if self.halted {
            return Ok(());
        }

        if self.breakpoints.contains(&self.pc) {
            self.breakpoints.remove(&self.pc);
            error!("Breakpoint");
            return Err(());
        }

        let (instruction, len) = self.fetch_instruction()?;
        if self.last_instructions.len() > 50 {
            self.last_instructions.pop_front();
        }
        self.last_instructions
            .push_back((self.mmu.cart.map_address_into_rom(self.pc), instruction));

        self.pc += Address(u16::from(len));
        self.execute(instruction)?;

        self.drive_peripherals()
    }

    pub fn run_for_duration(&mut self, duration: &Duration) {
        let cycles_to_run = duration_to_cycle_count(&duration);
        let stop_at_cycle = self.cycle() + cycles_to_run;
        self.mmu
            .lcd
            .set_running_until(stop_at_cycle + LONGEST_INSTRUCTION_CYCLE);
        while self.cycle() < stop_at_cycle && !self.debug_halted {
            if self.run_cycle().is_err() {
                self.debug_halted = true;
            }

            if self.halted {
                self.cycle = min(
                    self.mmu.lcd.get_next_event_cycle(),
                    min(self.mmu.timer.get_next_event_cycle(), stop_at_cycle),
                );
                if self.drive_peripherals().is_err() {
                    self.debug_halted = true;
                }
            }
        }
    }

    fn drive_peripherals(&mut self) -> Result<(), ()> {
        if let Some(i) = self.mmu.lcd.pump_cycle(self.cycle) {
            self.request_interrupt(i);
        }
        if let Some(i) = self.mmu.timer.pump_cycle(self.cycle) {
            self.request_interrupt(i);
        }
        Ok(())
    }

    fn request_interrupt(&mut self, int: Interrupt) {
        self.mmu.interrupt_flag |= int.bits();
        if int.bits() & self.mmu.interrupt_enable != 0 {
            self.halted = false;
        }
    }

    fn fire_interrupts(&mut self) -> Result<(), ()> {
        if self.interrupt_master_enable {
            if let (Some(int), if_) =
                Interrupt::int_to_run(self.mmu.interrupt_flag, self.mmu.interrupt_enable)
            {
                self.mmu.interrupt_flag = if_;
                self.fire_interrupt(int)?;
                if self.interrupt_breakpoints.contains(&int) {
                    self.interrupt_breakpoints.remove(&int);
                    debug!("Interrupt breakpoint {:?}", int);
                    self.debug_halted = true;
                }
            }
        }

        Ok(())
    }

    fn fire_interrupt(&mut self, int: Interrupt) -> Result<(), ()> {
        let v = self.pc.into();
        self.push16(v)?;

        self.pc = int.table_address();
        self.interrupt_master_enable = false;

        Ok(())
    }

    pub fn fetch_instruction(&self) -> Result<(Instruction, u8), ()> {
        let bytes = [
            self.mmu.read(self.pc)?,
            self.mmu.read(self.pc + Address(1))?,
            self.mmu.read(self.pc + Address(2))?,
        ];
        Instruction::decode(bytes)
    }

    fn write_r16(&mut self, r: Register16, v: u16) {
        match r {
            Register16::SP => self.sp = Address(v),
            Register16::PC => self.pc = Address(v),
            Register16::AF => {
                self[Register8::A] = hi(v);
                self[Register8::F] = lo(v) & 0xF0;
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
        self.mmu.write16(nsp, v)?;
        self.sp = nsp;
        Ok(())
    }

    fn pop16(&mut self) -> Result<u16, ()> {
        let v = self.mmu.read16(self.sp)?;
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

    pub fn request_p1_int(&mut self) {
        self.mmu.interrupt_flag |= Interrupt::Controller.bits();
    }
}

pub fn duration_to_cycle_count(duration: &Duration) -> u64 {
    // Clock for the CPU is 4.19 MHz
    const NSEC_PER_SEC: u64 = 1_000_000_000;
    let scount = duration.as_secs() * CLOCK_RATE;
    let ncount = CLOCK_RATE * u64::from(duration.subsec_nanos()) / NSEC_PER_SEC;
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
