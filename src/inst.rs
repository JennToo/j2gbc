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
    CpI(u8),
    JrNZI(i8),
    AndI(/* will always love */ u8),
    Ret,
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
            Instruction::CpI(_) => 8,
            // TODO: This is actually variable
            Instruction::JrNZI(_) => 8,
            Instruction::AndI(_) => 8,
            Instruction::Ret => 16,
        }
    }

    pub fn decode(bytes: [u8; 3]) -> Result<(Instruction, u8), ()> {
        match bytes[0] {
            0 => Ok((Instruction::Nop, 1)),
            0xC3 => Ok((Instruction::JpI(Address(hi_lo(bytes[2], bytes[1]))), 3)),
            0xCD => Ok((Instruction::CallI(Address(hi_lo(bytes[2], bytes[1]))), 3)),
            0xF0 => Ok((
                Instruction::LdRM(Register8::A, Address(0xFF00) + Address(bytes[1] as u16)),
                2,
            )),
            0xE0 => Ok((
                Instruction::LdMR(Address(0xFF00) + Address(bytes[1] as u16), Register8::A),
                2,
            )),
            0xFE => Ok((Instruction::CpI(bytes[1]), 2)),
            0x20 => Ok((Instruction::JrNZI(bytes[1] as i8), 2)),
            0xE6 => Ok((Instruction::AndI(bytes[1]), 2)),
            0xC9 => Ok((Instruction::Ret, 1)),
            0xCB => {
                match bytes[1] {
                    0x87 => Ok((Instruction::Res(0, Register8::A), 2)),
                    _ => {
                        println!(
                            "Unknown instruction {:#X} {:#X} {:#X}",
                            bytes[0], bytes[1], bytes[2]
                        );
                        Err(())
                    },
                }
            }
            _ => {
                println!(
                    "Unknown instruction {:#X} {:#X} {:#X}",
                    bytes[0], bytes[1], bytes[2]
                );
                Err(())
            }
        }
    }
}
