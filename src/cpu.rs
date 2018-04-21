use std::ops::{Index, IndexMut};
use std::num::Wrapping;

use alu::{and, dec, hi, hi_lo, inc, lo, or, sub, xor, Flags, add16, MASK_FLAG_Z};
use inst::{Arith, Control, Instruction, Load, Logic};
use mem::{Address, MemDevice, Mmu};
use cart::Cart;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    F,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

pub struct Cpu {
    registers: [u8; 8],
    pub pc: Address,
    pub sp: Address,
    mmu: Mmu,
    cycle: u64,
}

impl Cpu {
    pub fn new(c: Cart) -> Cpu {
        Cpu {
            registers: [0, 0, 0, 0, 0, 0, 0, 0],
            sp: Address(0xFFFE),
            pc: Address(0x100),
            mmu: Mmu::new(c),
            cycle: 0,
        }
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    fn execute(&mut self, i: Instruction) -> Result<(), ()> {
        match i {
            Instruction::Nop => {}
            Instruction::Res(b, r) => {
                let v = self[r] & !(1 << b);
                self[r] = v;
            }
            Instruction::CpI(v) => {
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
            Arith::IncR(r) => {
                let f = Flags(self[Register8::F]);
                let (v, flags) = inc(self[r], f);
                self[r] = v;
                self[Register8::F] = flags.0;
            }
            Arith::DecR(r) => {
                let f = Flags(self[Register8::F]);
                let (v, flags) = dec(self[r], f);
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
                let f = Flags(self[Register8::F]);
                let v1 = self.read_r16(d);
                let v2 = self.read_r16(s);
                let (v3, flags) = add16(v1, v2, f);
                self.write_r16(d, v3);
                self[Register8::F] = flags.0;
            }
        }
        Ok(())
    }

    fn execute_control(&mut self, c: Control) -> Result<(), ()> {
        match c {
            Control::JrNZI(o) => {
                if self[Register8::F] & MASK_FLAG_Z == 0 {
                    self.pc = Address((self.pc.0 as i32 + o as i32) as u16);
                }
            }
            Control::Ret => {
                self.pc = Address(try!(self.mmu.read16(self.sp)));
                self.sp += Address(2);
            }
            Control::JpI(a) => {
                self.pc = a;
            }
            Control::CallI(a) => {
                let nsp = self.sp - Address(2);
                try!(self.mmu.write16(nsp, self.pc.into()));
                self.sp = nsp;
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
            Load::LdNR16(r) => {
                let v = self[Register8::A];
                let a = Address(self.read_r16(r));
                try!(self.mmu.write(a, v));
            }
            Load::LdRN16(r) => {
                let a = Address(self.read_r16(r));
                let v = try!(self.mmu.read(a));
                self[Register8::A] = v;
            }
            Load::LdRI16(r, i) => {
                self.write_r16(r, i);
            }
            Load::LdNI16(a) => {
                let v = self[Register8::A];
                try!(self.mmu.write(a, v));
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
            Logic::OrR(r) => {
                let (value, flags) = or(self[Register8::A], self[r]);
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
        let (instruction, len) = try!(self.fetch_instruction());
        println!("Running {:?}: {:?}", self.pc, instruction);
        self.pc += Address(len as u16);
        self.execute(instruction)
    }

    fn fetch_instruction(&self) -> Result<(Instruction, u8), ()> {
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
