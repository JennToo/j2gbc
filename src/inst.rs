use mem::Address;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    JpI(Address),
    CallI(Address),
}

fn hi_lo(hi: u8, lo: u8) -> u16 {
    (hi as u16) << 8 | lo as u16
}

impl Instruction {
    pub fn cycles(self) -> u8 {
        match self {
            Instruction::Nop => 4,
            Instruction::JpI(_) => 16,
            Instruction::CallI(_) => 24,
        }
    }

    pub fn decode(bytes: [u8; 3]) -> Instruction {
        match bytes[0] {
            0 => Instruction::Nop,
            0xC3 => Instruction::JpI(Address(hi_lo(bytes[2], bytes[1]))),
            0xCD => Instruction::CallI(Address(hi_lo(bytes[2], bytes[1]))),
            _ => panic!("Unknown instruction {:#X} {:#X} {:#X}", bytes[0], bytes[1], bytes[2]),
        }
    }
}
