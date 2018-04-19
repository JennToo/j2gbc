use mem::Address;
use cpu::{Register16, Register8};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    Res(u8, Register8),
    CpI(u8),
    Control(Control),
    Load(Load),
    Logic(Logic),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Control {
    JrNZI(i8),
    Ret,
    JpI(Address),
    CallI(Address),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Load {
    LdRM(Register8, Address),
    LdMR(Address, Register8),
    LdRR(Register8, Register8),
    LdRI16(Register16, u16),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Logic {
    AndI(/* will always love */ u8),
    XorR(Register8),
}

fn hi_lo(hi: u8, lo: u8) -> u16 {
    (hi as u16) << 8 | lo as u16
}

impl Instruction {
    pub fn cycles(self) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Instruction::Nop => 4,
            Instruction::Res(_, _) => 8,
            Instruction::CpI(_) => 8,
            Instruction::Load(l) => l.cycles(),
            Instruction::Control(c) => c.cycles(),
            Instruction::Logic(l) => l.cycles(),
        }
    }

    pub fn decode(bytes: [u8; 3]) -> Result<(Instruction, u8), ()> {
        match bytes[0] {
            0 => Ok((Instruction::Nop, 1)),
            0xC3 => Ok((
                Instruction::Control(Control::JpI(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xCD => Ok((
                Instruction::Control(Control::CallI(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xF0 => Ok((
                Instruction::Load(Load::LdRM(
                    Register8::A,
                    Address(0xFF00) + Address(bytes[1] as u16),
                )),
                2,
            )),
            0xE0 => Ok((
                Instruction::Load(Load::LdMR(
                    Address(0xFF00) + Address(bytes[1] as u16),
                    Register8::A,
                )),
                2,
            )),
            0x01 => Ok((
                Instruction::Load(Load::LdRI16(Register16::BC, hi_lo(bytes[2], bytes[1]))),
                3,
            )),
            0x11 => Ok((
                Instruction::Load(Load::LdRI16(Register16::DE, hi_lo(bytes[2], bytes[1]))),
                3,
            )),
            0x21 => Ok((
                Instruction::Load(Load::LdRI16(Register16::HL, hi_lo(bytes[2], bytes[1]))),
                3,
            )),
            0x31 => Ok((
                Instruction::Load(Load::LdRI16(Register16::SP, hi_lo(bytes[2], bytes[1]))),
                3,
            )),
            0x47 => Ok((Instruction::Load(Load::LdRR(Register8::B, Register8::A)), 1)),
            0xFE => Ok((Instruction::CpI(bytes[1]), 2)),
            0x20 => Ok((Instruction::Control(Control::JrNZI(bytes[1] as i8)), 2)),
            0xE6 => Ok((Instruction::Logic(Logic::AndI(bytes[1])), 2)),
            0xAF => Ok((Instruction::Logic(Logic::XorR(Register8::A)), 2)),
            0xC9 => Ok((Instruction::Control(Control::Ret), 1)),
            0xCB => match bytes[1] {
                0x87 => Ok((Instruction::Res(0, Register8::A), 2)),
                _ => {
                    println!(
                        "Unknown instruction {:#X} {:#X} {:#X}",
                        bytes[0], bytes[1], bytes[2]
                    );
                    Err(())
                }
            },
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

impl Control {
    fn cycles(self) -> u8 {
        match self {
            Control::JpI(_) => 16,
            Control::CallI(_) => 24,
            // TODO: This is actually variable
            Control::JrNZI(_) => 8,
            Control::Ret => 16,
        }
    }
}

impl Load {
    fn cycles(self) -> u8 {
        match self {
            Load::LdRM(_, _) | Load::LdRI16(_, _) | Load::LdMR(_, _) => 12,
            Load::LdRR(_, _) => 4,
        }
    }
}

impl Logic {
    fn cycles(self) -> u8 {
        match self {
            Logic::AndI(_) => 8,
            Logic::XorR(_) => 4,
        }
    }
}
