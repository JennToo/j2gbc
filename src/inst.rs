use alu::hi_lo;
use mem::Address;
use cpu::{Register16, Register8};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    Ei,
    Di,
    Res(u8, Register8),
    CpI(u8),
    CpR(Register8),
    Arith(Arith),
    Control(Control),
    Load(Load),
    Logic(Logic),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Arith {
    IncR(Register8),
    DecR(Register8),
    IncR16(Register16),
    DecR16(Register16),
    AddRR16(Register16, Register16),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Control {
    JrNZI(i8),
    JrI(i8),
    JrNCI(i8),
    JrZI(i8),
    JrCI(i8),
    Ret,
    JpI(Address),
    JpCI(Address),
    JpZI(Address),
    JpNCI(Address),
    JpNZI(Address),
    CallI(Address),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Load {
    LdRM(Register8, Address),
    LdMR(Address, Register8),
    LdRR(Register8, Register8),
    LdRI(Register8, u8),
    LdNA(i8),
    LdAN(i8),
    LdNI(u8),
    LdNCA,
    LdANC,
    LdRI16(Register16, u16),
    LdNIA16(Address),
    LdANI16(Address),
    LdNR16(Register16),
    LdRN16(Register16),
    Push(Register16),
    Pop(Register16),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Logic {
    AndI(/* will always love */ u8),
    AndR(Register8),
    AndN,

    /* hallowed are the */ OrI(u8),
    OrR(Register8),
    OrN,

    XorR(Register8),
}

impl Instruction {
    pub fn cycles(self) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Instruction::Nop => 4,
            Instruction::Ei => 4,
            Instruction::Di => 4,
            Instruction::Res(_, _) => 8,
            Instruction::CpI(_) => 8,
            Instruction::CpR(_) => 4,
            Instruction::Arith(a) => a.cycles(),
            Instruction::Load(l) => l.cycles(),
            Instruction::Control(c) => c.cycles(),
            Instruction::Logic(l) => l.cycles(),
        }
    }

    pub fn decode(bytes: [u8; 3]) -> Result<(Instruction, u8), ()> {
        match bytes[0] {
            0 => Ok((Instruction::Nop, 1)),

            0xFB => Ok((Instruction::Ei, 1)),
            0xF3 => Ok((Instruction::Di, 1)),

            0x04 => Ok((Instruction::Arith(Arith::IncR(Register8::B)), 1)),
            0x14 => Ok((Instruction::Arith(Arith::IncR(Register8::D)), 1)),
            0x24 => Ok((Instruction::Arith(Arith::IncR(Register8::H)), 1)),
            0x0C => Ok((Instruction::Arith(Arith::IncR(Register8::C)), 1)),
            0x1C => Ok((Instruction::Arith(Arith::IncR(Register8::E)), 1)),
            0x2C => Ok((Instruction::Arith(Arith::IncR(Register8::L)), 1)),
            0x3C => Ok((Instruction::Arith(Arith::IncR(Register8::A)), 1)),

            0x05 => Ok((Instruction::Arith(Arith::DecR(Register8::B)), 1)),
            0x15 => Ok((Instruction::Arith(Arith::DecR(Register8::D)), 1)),
            0x25 => Ok((Instruction::Arith(Arith::DecR(Register8::H)), 1)),
            0x0D => Ok((Instruction::Arith(Arith::DecR(Register8::C)), 1)),
            0x1D => Ok((Instruction::Arith(Arith::DecR(Register8::E)), 1)),
            0x2D => Ok((Instruction::Arith(Arith::DecR(Register8::L)), 1)),
            0x3D => Ok((Instruction::Arith(Arith::DecR(Register8::A)), 1)),

            0x0B => Ok((Instruction::Arith(Arith::DecR16(Register16::BC)), 1)),
            0x1B => Ok((Instruction::Arith(Arith::DecR16(Register16::DE)), 1)),
            0x2B => Ok((Instruction::Arith(Arith::DecR16(Register16::HL)), 1)),
            0x3B => Ok((Instruction::Arith(Arith::DecR16(Register16::SP)), 1)),

            0x03 => Ok((Instruction::Arith(Arith::IncR16(Register16::BC)), 1)),
            0x13 => Ok((Instruction::Arith(Arith::IncR16(Register16::DE)), 1)),
            0x23 => Ok((Instruction::Arith(Arith::IncR16(Register16::HL)), 1)),
            0x33 => Ok((Instruction::Arith(Arith::IncR16(Register16::SP)), 1)),

            0x09 => Ok((
                Instruction::Arith(Arith::AddRR16(Register16::HL, Register16::BC)),
                1,
            )),
            0x19 => Ok((
                Instruction::Arith(Arith::AddRR16(Register16::HL, Register16::DE)),
                1,
            )),
            0x29 => Ok((
                Instruction::Arith(Arith::AddRR16(Register16::HL, Register16::HL)),
                1,
            )),
            0x39 => Ok((
                Instruction::Arith(Arith::AddRR16(Register16::HL, Register16::SP)),
                1,
            )),

            0xC3 => Ok((
                Instruction::Control(Control::JpI(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xC2 => Ok((
                Instruction::Control(Control::JpNZI(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xD2 => Ok((
                Instruction::Control(Control::JpCI(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xCA => Ok((
                Instruction::Control(Control::JpZI(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xDA => Ok((
                Instruction::Control(Control::JpCI(Address(hi_lo(bytes[2], bytes[1])))),
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

            0xEA => Ok((
                Instruction::Load(Load::LdNIA16(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xFA => Ok((
                Instruction::Load(Load::LdANI16(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),

            0xE2 => Ok((Instruction::Load(Load::LdNCA), 1)),
            0xF2 => Ok((Instruction::Load(Load::LdANC), 1)),

            0x36 => Ok((Instruction::Load(Load::LdNI(bytes[1])), 2)),

            0x02 => Ok((Instruction::Load(Load::LdNR16(Register16::BC)), 1)),
            0x12 => Ok((Instruction::Load(Load::LdNR16(Register16::DE)), 1)),

            0x0A => Ok((Instruction::Load(Load::LdRN16(Register16::BC)), 1)),
            0x1A => Ok((Instruction::Load(Load::LdRN16(Register16::DE)), 1)),

            0x06 => Ok((Instruction::Load(Load::LdRI(Register8::B, bytes[1])), 2)),
            0x16 => Ok((Instruction::Load(Load::LdRI(Register8::D, bytes[1])), 2)),
            0x26 => Ok((Instruction::Load(Load::LdRI(Register8::H, bytes[1])), 2)),
            0x0E => Ok((Instruction::Load(Load::LdRI(Register8::C, bytes[1])), 2)),
            0x1E => Ok((Instruction::Load(Load::LdRI(Register8::E, bytes[1])), 2)),
            0x2E => Ok((Instruction::Load(Load::LdRI(Register8::L, bytes[1])), 2)),
            0x3E => Ok((Instruction::Load(Load::LdRI(Register8::A, bytes[1])), 2)),

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

            0x22 => Ok((Instruction::Load(Load::LdNA(1)), 1)),
            0x32 => Ok((Instruction::Load(Load::LdNA(-1)), 1)),

            0x2A => Ok((Instruction::Load(Load::LdAN(1)), 1)),
            0x3A => Ok((Instruction::Load(Load::LdAN(-1)), 1)),

            0xC5 => Ok((Instruction::Load(Load::Push(Register16::BC)), 1)),
            0xD5 => Ok((Instruction::Load(Load::Push(Register16::DE)), 1)),
            0xE5 => Ok((Instruction::Load(Load::Push(Register16::HL)), 1)),
            0xF5 => Ok((Instruction::Load(Load::Push(Register16::AF)), 1)),

            0xC1 => Ok((Instruction::Load(Load::Pop(Register16::BC)), 1)),
            0xD1 => Ok((Instruction::Load(Load::Pop(Register16::DE)), 1)),
            0xE1 => Ok((Instruction::Load(Load::Pop(Register16::HL)), 1)),
            0xF1 => Ok((Instruction::Load(Load::Pop(Register16::AF)), 1)),

            0xFE => Ok((Instruction::CpI(bytes[1]), 2)),

            0xB8 => Ok((Instruction::CpR(Register8::B), 1)),
            0xB9 => Ok((Instruction::CpR(Register8::C), 1)),
            0xBA => Ok((Instruction::CpR(Register8::D), 1)),
            0xBB => Ok((Instruction::CpR(Register8::E), 1)),
            0xBC => Ok((Instruction::CpR(Register8::H), 1)),
            0xBD => Ok((Instruction::CpR(Register8::L), 1)),
            0xBF => Ok((Instruction::CpR(Register8::A), 1)),

            0x20 => Ok((Instruction::Control(Control::JrNZI(bytes[1] as i8)), 2)),
            0x30 => Ok((Instruction::Control(Control::JrNCI(bytes[1] as i8)), 2)),
            0x18 => Ok((Instruction::Control(Control::JrI(bytes[1] as i8)), 2)),
            0x28 => Ok((Instruction::Control(Control::JrZI(bytes[1] as i8)), 2)),
            0x38 => Ok((Instruction::Control(Control::JrCI(bytes[1] as i8)), 2)),

            0xE6 => Ok((Instruction::Logic(Logic::AndI(bytes[1])), 2)),
            0xA6 => Ok((Instruction::Logic(Logic::AndN), 1)),

            0xA0 => Ok((Instruction::Logic(Logic::AndR(Register8::B)), 1)),
            0xA1 => Ok((Instruction::Logic(Logic::AndR(Register8::C)), 1)),
            0xA2 => Ok((Instruction::Logic(Logic::AndR(Register8::D)), 1)),
            0xA3 => Ok((Instruction::Logic(Logic::AndR(Register8::E)), 1)),
            0xA4 => Ok((Instruction::Logic(Logic::AndR(Register8::H)), 1)),
            0xA5 => Ok((Instruction::Logic(Logic::AndR(Register8::L)), 1)),
            0xA7 => Ok((Instruction::Logic(Logic::AndR(Register8::A)), 1)),

            0xF6 => Ok((Instruction::Logic(Logic::OrI(bytes[1])), 2)),
            0xB6 => Ok((Instruction::Logic(Logic::OrN), 1)),

            0xB0 => Ok((Instruction::Logic(Logic::OrR(Register8::B)), 1)),
            0xB1 => Ok((Instruction::Logic(Logic::OrR(Register8::C)), 1)),
            0xB2 => Ok((Instruction::Logic(Logic::OrR(Register8::D)), 1)),
            0xB3 => Ok((Instruction::Logic(Logic::OrR(Register8::E)), 1)),
            0xB4 => Ok((Instruction::Logic(Logic::OrR(Register8::H)), 1)),
            0xB5 => Ok((Instruction::Logic(Logic::OrR(Register8::L)), 1)),

            0xAF => Ok((Instruction::Logic(Logic::XorR(Register8::A)), 1)),
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
            Arith::DecR16(_) | Arith::IncR16(_) => 8,
            Arith::IncR(_) | Arith::DecR(_) => 4,
            Arith::AddRR16(_, _) => 16,
        }
    }
}

impl Control {
    fn cycles(self) -> u8 {
        match self {
            Control::JpCI(_)
            | Control::JpI(_)
            | Control::JpZI(_)
            | Control::JpNCI(_)
            | Control::JpNZI(_) => 16,
            Control::CallI(_) => 24,
            // TODO: This is actually variable
            Control::JrNZI(_)
            | Control::JrNCI(_)
            | Control::JrI(_)
            | Control::JrCI(_)
            | Control::JrZI(_) => 12,
            Control::Ret => 16,
        }
    }
}

impl Load {
    fn cycles(self) -> u8 {
        match self {
            Load::LdRM(_, _) | Load::LdRI16(_, _) | Load::LdMR(_, _) => 12,
            Load::LdRR(_, _) => 4,
            Load::LdRI(_, _) => 8,
            Load::LdNA(_) | Load::LdAN(_) => 8,
            Load::LdNI(_) => 12,
            Load::LdNR16(_) | Load::LdRN16(_) => 8,
            Load::LdNIA16(_) | Load::LdANI16(_) => 16,
            Load::LdNCA => 8,
            Load::LdANC => 8,
            Load::Push(_) => 16,
            Load::Pop(_) => 12,
        }
    }
}

impl Logic {
    fn cycles(self) -> u8 {
        match self {
            Logic::AndI(_) | Logic::OrI(_) => 8,
            Logic::XorR(_) | Logic::AndR(_) | Logic::OrR(_) => 4,
            Logic::AndN | Logic::OrN => 8,
        }
    }
}
