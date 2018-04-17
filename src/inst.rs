use mem::Address;
use cpu::Register8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    JpI(Address),
    CallI(Address),
    LdRM(Register8, Address),
    LdMR(Address, Register8),
    Res(u8, Register8),
}

fn hi_lo(hi: u8, lo: u8) -> u16 {
    (hi as u16) << 8 | lo as u16
}

impl Instruction {
    pub fn cycles(self) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Instruction::Nop => 4,
            Instruction::JpI(_) => 16,
            Instruction::CallI(_) => 24,
            Instruction::LdRM(_, _) => 12,
            Instruction::LdMR(_, _) => 12,
            Instruction::Res(_, _) => 8,
        }
    }

    pub fn decode(bytes: [u8; 3]) -> (Instruction, u8) {
        match bytes[0] {
            0 => (Instruction::Nop, 1),
            0xC3 => (Instruction::JpI(Address(hi_lo(bytes[2], bytes[1]))), 3),
            0xCD => (Instruction::CallI(Address(hi_lo(bytes[2], bytes[1]))), 3),
            0xF0 => (
                Instruction::LdRM(Register8::A, Address(0xFF00) + Address(bytes[1] as u16)),
                2,
            ),
            0xE0 => (
                Instruction::LdMR(Address(0xFF00) + Address(bytes[1] as u16), Register8::A),
                2,
            ),
            0xCB => (
                match bytes[1] {
                    0x87 => Instruction::Res(0, Register8::A),
                    _ => panic!(
                        "Unknown instruction {:#X} {:#X} {:#X}",
                        bytes[0], bytes[1], bytes[2]
                    ),
                },
                2,
            ),
            _ => panic!(
                "Unknown instruction {:#X} {:#X} {:#X}",
                bytes[0], bytes[1], bytes[2]
            ),
        }
    }
}
