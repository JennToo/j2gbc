use std::ops::{Index, IndexMut};

use inst::Instruction;
use mem::{Address, Mmu, MemDevice};
use cart::Cart;

#[repr(u8)]
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
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC
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
                self.mmu.write16(self.sp, self.pc.0);
                self.sp.0 -= 2;
                self.pc = a;
            }
        }        
        self.cycle += i.cycles() as u64;
    }

    pub fn run_cycle(&mut self) {
        let instruction = self.fetch_instruction();
        println!("Running {:#X}: {:?}", self.pc.0, instruction);
        self.pc.0 += 1;
        self.execute(instruction);
    }

    fn fetch_instruction(&self) -> Instruction {
        let bytes = [self.mmu.read(self.pc), self.mmu.read(Address(self.pc.0 + 1)), self.mmu.read(Address(self.pc.0 + 2))];
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
