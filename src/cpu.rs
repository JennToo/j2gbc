use std::ops::{Index, IndexMut};
use std::num::Wrapping;

use inst::Instruction;
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
    pc: Address,
    sp: Address,
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

    fn execute(&mut self, i: Instruction) {
        match i {
            Instruction::Nop => {}
            Instruction::JpI(a) => {
                self.pc = a;
            }
            Instruction::CallI(a) => {
                let nsp = self.sp - Address(2);
                self.mmu.write16(self.sp, nsp.into());
                self.sp = nsp;
                self.pc = a;
            }
            Instruction::LdRM(r, a) => {
                self[r] = self.mmu.read(a);
            }
            Instruction::LdMR(a, r) => {
                let v = self[r];
                self.mmu.write(a, v);
            }
            Instruction::Res(b, r) => {
                let v = self[r] & !(1 << b);
                self[r] = v;
            }
            Instruction::CpI(v) => {
                let (flags, _) = sub(self[Register8::A], v);
                self[Register8::F] = flags;
            }
            Instruction::JrNZI(o) => {
                if self[Register8::F] & MASK_FLAG_Z == 0 {
                    self.pc = Address((self.pc.0 as i32 + o as i32) as u16);
                }
            }
        }
        self.cycle += i.cycles() as u64;
    }

    pub fn run_cycle(&mut self) {
        let (instruction, len) = self.fetch_instruction();
        println!("Running {:?}: {:?}", self.pc, instruction);
        self.pc += Address(len as u16);
        self.execute(instruction);
    }

    fn fetch_instruction(&self) -> (Instruction, u8) {
        let bytes = [
            self.mmu.read(self.pc),
            self.mmu.read(self.pc + Address(1)),
            self.mmu.read(self.pc + Address(2)),
        ];
        Instruction::decode(bytes)
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
