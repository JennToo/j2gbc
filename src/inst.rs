use alu::hi_lo;
use mem::Address;
use cpu::{Register16, Register8};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    Res(u8, Register8),
    CpI(u8),
    Arith(Arith),
    Control(Control),
    Load(Load),
    Logic(Logic),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Arith {
    DecR16(Register16),
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
    OrR(Register8),
    XorR(Register8),
}

impl Instruction {
    pub fn cycles(self) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Instruction::Nop => 4,
            Instruction::Res(_, _) => 8,
            Instruction::CpI(_) => 8,
            Instruction::Arith(a) => a.cycles(),
            Instruction::Load(l) => l.cycles(),
            Instruction::Control(c) => c.cycles(),
            Instruction::Logic(l) => l.cycles(),
        }
    }

    pub fn decode(bytes: [u8; 3]) -> Result<(Instruction, u8), ()> {
        match bytes[0] {
            0 => Ok((Instruction::Nop, 1)),
            0x0B => Ok((Instruction::Arith(Arith::DecR16(Register16::BC)), 1)),
            0x1B => Ok((Instruction::Arith(Arith::DecR16(Register16::DE)), 1)),
            0x2B => Ok((Instruction::Arith(Arith::DecR16(Register16::HL)), 1)),
            0x3B => Ok((Instruction::Arith(Arith::DecR16(Register16::SP)), 1)),
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

            0x40 => Ok((Instruction::Load(Load::LdRR(Register8::B, Register8::B)), 1)),
            0x41 => Ok((Instruction::Load(Load::LdRR(Register8::B, Register8::C)), 1)),
            0x42 => Ok((Instruction::Load(Load::LdRR(Register8::B, Register8::D)), 1)),
            0x43 => Ok((Instruction::Load(Load::LdRR(Register8::B, Register8::H)), 1)),
            0x44 => Ok((Instruction::Load(Load::LdRR(Register8::B, Register8::L)), 1)),
            0x47 => Ok((Instruction::Load(Load::LdRR(Register8::B, Register8::A)), 1)),
            0x48 => Ok((Instruction::Load(Load::LdRR(Register8::C, Register8::B)), 1)),
            0x49 => Ok((Instruction::Load(Load::LdRR(Register8::C, Register8::C)), 1)),
            0x4A => Ok((Instruction::Load(Load::LdRR(Register8::C, Register8::D)), 1)),
            0x4B => Ok((Instruction::Load(Load::LdRR(Register8::C, Register8::E)), 1)),
            0x4C => Ok((Instruction::Load(Load::LdRR(Register8::C, Register8::H)), 1)),
            0x4D => Ok((Instruction::Load(Load::LdRR(Register8::C, Register8::L)), 1)),
            0x50 => Ok((Instruction::Load(Load::LdRR(Register8::D, Register8::B)), 1)),
            0x51 => Ok((Instruction::Load(Load::LdRR(Register8::D, Register8::C)), 1)),
            0x52 => Ok((Instruction::Load(Load::LdRR(Register8::D, Register8::D)), 1)),
            0x53 => Ok((Instruction::Load(Load::LdRR(Register8::D, Register8::H)), 1)),
            0x54 => Ok((Instruction::Load(Load::LdRR(Register8::D, Register8::L)), 1)),
            0x57 => Ok((Instruction::Load(Load::LdRR(Register8::D, Register8::A)), 1)),
            0x58 => Ok((Instruction::Load(Load::LdRR(Register8::E, Register8::B)), 1)),
            0x59 => Ok((Instruction::Load(Load::LdRR(Register8::E, Register8::C)), 1)),
            0x5A => Ok((Instruction::Load(Load::LdRR(Register8::E, Register8::D)), 1)),
            0x5B => Ok((Instruction::Load(Load::LdRR(Register8::E, Register8::E)), 1)),
            0x5C => Ok((Instruction::Load(Load::LdRR(Register8::E, Register8::H)), 1)),
            0x5D => Ok((Instruction::Load(Load::LdRR(Register8::E, Register8::L)), 1)),
            0x60 => Ok((Instruction::Load(Load::LdRR(Register8::H, Register8::B)), 1)),
            0x61 => Ok((Instruction::Load(Load::LdRR(Register8::H, Register8::C)), 1)),
            0x62 => Ok((Instruction::Load(Load::LdRR(Register8::H, Register8::D)), 1)),
            0x63 => Ok((Instruction::Load(Load::LdRR(Register8::H, Register8::H)), 1)),
            0x64 => Ok((Instruction::Load(Load::LdRR(Register8::H, Register8::L)), 1)),
            0x67 => Ok((Instruction::Load(Load::LdRR(Register8::H, Register8::A)), 1)),
            0x68 => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::B)), 1)),
            0x69 => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::C)), 1)),
            0x6A => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::D)), 1)),
            0x6B => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::E)), 1)),
            0x6C => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::H)), 1)),
            0x6D => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::L)), 1)),
            0x78 => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::B)), 1)),
            0x79 => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::C)), 1)),
            0x7A => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::D)), 1)),
            0x7B => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::E)), 1)),
            0x7C => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::H)), 1)),
            0x7D => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::L)), 1)),

            0xFE => Ok((Instruction::CpI(bytes[1]), 2)),
            0x20 => Ok((Instruction::Control(Control::JrNZI(bytes[1] as i8)), 2)),
            0xE6 => Ok((Instruction::Logic(Logic::AndI(bytes[1])), 2)),

            0xB0 => Ok((Instruction::Logic(Logic::OrR(Register8::B)), 1)),
            0xB1 => Ok((Instruction::Logic(Logic::OrR(Register8::C)), 1)),
            0xB2 => Ok((Instruction::Logic(Logic::OrR(Register8::D)), 1)),
            0xB3 => Ok((Instruction::Logic(Logic::OrR(Register8::E)), 1)),
            0xB4 => Ok((Instruction::Logic(Logic::OrR(Register8::H)), 1)),
            0xB5 => Ok((Instruction::Logic(Logic::OrR(Register8::L)), 1)),

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

impl Arith {
    fn cycles(self) -> u8 {
        match self {
            Arith::DecR16(_) => 8,
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
            Logic::OrR(_) => 4,
        }
    }
}
