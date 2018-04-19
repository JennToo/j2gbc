use std::ops::{Index, IndexMut};
use std::num::Wrapping;

use inst::{Control, Instruction, Load, Logic};
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
                let (flags, _) = sub(self[Register8::A], v);
                self[Register8::F] = flags;
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
            Load::LdRI16(r, i) => {
                self.write_r16(r, i);
            }
            Load::LdRR(d, s) => {
                let v = self[s];
                self[d] = v;
            }
        }

        Ok(())
    }

    fn execute_logic(&mut self, l: Logic) -> Result<(), ()> {
        match l {
            Logic::AndI(v) => {
                let (flags, value) = and(self[Register8::A], v);
                self[Register8::A] = value;
                self[Register8::F] = flags;
            }
            Logic::XorR(r) => {
                let (flags, value) = xor(self[Register8::A], self[r]);
                self[Register8::A] = value;
                self[Register8::F] = flags;
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
}

pub fn hi(v: u16) -> u8 {
    ((v >> 8) & 0xFF) as u8
}

pub fn lo(v: u16) -> u8 {
    (v & 0xFF) as u8
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

const MASK_FLAG_Z: u8 = 0b1000_0000;
const MASK_FLAG_N: u8 = 0b0100_0000;
const MASK_FLAG_H: u8 = 0b0010_0000;
const MASK_FLAG_C: u8 = 0b0001_0000;

fn sub(l: u8, r: u8) -> (u8, Wrapping<u8>) {
    if l < r {
        ((MASK_FLAG_N | MASK_FLAG_C), Wrapping(l) - Wrapping(r))
    } else if l > r {
        ((MASK_FLAG_N | MASK_FLAG_H), Wrapping(l) - Wrapping(r))
    } else {
        ((MASK_FLAG_N | MASK_FLAG_Z), Wrapping(l) - Wrapping(r))
    }
}

fn and(l: u8, r: u8) -> (u8, u8) {
    let v = l & r;
    if v == 0 {
        ((MASK_FLAG_H | MASK_FLAG_Z), v)
    } else {
        (MASK_FLAG_H, v)
    }
}

fn xor(l: u8, r: u8) -> (u8, u8) {
    let v = l ^ r;
    if v == 0 {
        (MASK_FLAG_Z, v)
    } else {
        (0, v)
    }
}
