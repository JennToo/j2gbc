use std::fmt;
use std::fmt::Display;

use super::alu::hi_lo;
use super::cpu::{Register16, Register8};
use super::mem::Address;

mod arith;
mod bits;
mod control;
mod load;
mod logic;

pub use self::arith::Arith;
pub use self::bits::Bits;
pub use self::control::Control;
pub use self::load::Load;
pub use self::logic::Logic;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    Ei,
    Di,
    Halt,
    Scf,
    CpI(u8),
    CpR(Register8),
    Arith(Arith),
    Bits(Bits),
    Control(Control),
    Load(Load),
    Logic(Logic),
}

impl Instruction {
    pub fn cycles(self) -> u8 {
        // TODO: Audit this list for accuracy
        match self {
            Instruction::Nop => 4,
            Instruction::Ei => 4,
            Instruction::Di => 4,
            Instruction::Halt => 4,
            Instruction::Scf => 4,
            Instruction::CpI(_) => 8,
            Instruction::CpR(_) => 4,
            Instruction::Arith(a) => a.cycles(),
            Instruction::Bits(b) => b.cycles(),
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

            0x76 => Ok((Instruction::Halt, 1)),

            0x37 => Ok((Instruction::Scf, 1)),

            0x34 => Ok((Instruction::Arith(Arith::IncN), 1)),
            0x04 => Ok((Instruction::Arith(Arith::IncR(Register8::B)), 1)),
            0x14 => Ok((Instruction::Arith(Arith::IncR(Register8::D)), 1)),
            0x24 => Ok((Instruction::Arith(Arith::IncR(Register8::H)), 1)),
            0x0C => Ok((Instruction::Arith(Arith::IncR(Register8::C)), 1)),
            0x1C => Ok((Instruction::Arith(Arith::IncR(Register8::E)), 1)),
            0x2C => Ok((Instruction::Arith(Arith::IncR(Register8::L)), 1)),
            0x3C => Ok((Instruction::Arith(Arith::IncR(Register8::A)), 1)),

            0x35 => Ok((Instruction::Arith(Arith::DecN), 1)),
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

            0xE9 => Ok((Instruction::Control(Control::JpN), 1)),
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

            0xC7 => Ok((Instruction::Control(Control::Rst(Address(0x0000))), 1)),
            0xD7 => Ok((Instruction::Control(Control::Rst(Address(0x0010))), 1)),
            0xE7 => Ok((Instruction::Control(Control::Rst(Address(0x0020))), 1)),
            0xF7 => Ok((Instruction::Control(Control::Rst(Address(0x0030))), 1)),

            0xCF => Ok((Instruction::Control(Control::Rst(Address(0x0008))), 1)),
            0xDF => Ok((Instruction::Control(Control::Rst(Address(0x0018))), 1)),
            0xEF => Ok((Instruction::Control(Control::Rst(Address(0x0028))), 1)),
            0xFF => Ok((Instruction::Control(Control::Rst(Address(0x0038))), 1)),

            0xCD => Ok((
                Instruction::Control(Control::CallI(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xF0 => Ok((
                Instruction::Load(Load::LdRM(
                    Register8::A,
                    Address(0xFF00) + Address(u16::from(bytes[1])),
                )),
                2,
            )),
            0xE0 => Ok((
                Instruction::Load(Load::LdMR(
                    Address(0xFF00) + Address(u16::from(bytes[1])),
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

            0x2F => Ok((Instruction::Bits(Bits::Cpl), 1)),

            0x27 => Ok((Instruction::Arith(Arith::Daa), 1)),

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
            0x4F => Ok((Instruction::Load(Load::LdRR(Register8::C, Register8::A)), 1)),
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
            0x5F => Ok((Instruction::Load(Load::LdRR(Register8::E, Register8::A)), 1)),
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
            0x6F => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::A)), 1)),
            0x68 => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::B)), 1)),
            0x69 => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::C)), 1)),
            0x6A => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::D)), 1)),
            0x6B => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::E)), 1)),
            0x6C => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::H)), 1)),
            0x6D => Ok((Instruction::Load(Load::LdRR(Register8::L, Register8::L)), 1)),
            0x7F => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::A)), 1)),
            0x78 => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::B)), 1)),
            0x79 => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::C)), 1)),
            0x7A => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::D)), 1)),
            0x7B => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::E)), 1)),
            0x7C => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::H)), 1)),
            0x7D => Ok((Instruction::Load(Load::LdRR(Register8::A, Register8::L)), 1)),

            0x22 => Ok((Instruction::Load(Load::LdNA(1)), 1)),
            0x32 => Ok((Instruction::Load(Load::LdNA(-1)), 1)),

            0x46 => Ok((Instruction::Load(Load::LdRN(Register8::B)), 1)),
            0x56 => Ok((Instruction::Load(Load::LdRN(Register8::D)), 1)),
            0x66 => Ok((Instruction::Load(Load::LdRN(Register8::H)), 1)),
            0x4E => Ok((Instruction::Load(Load::LdRN(Register8::C)), 1)),
            0x5E => Ok((Instruction::Load(Load::LdRN(Register8::E)), 1)),
            0x6E => Ok((Instruction::Load(Load::LdRN(Register8::L)), 1)),
            0x7E => Ok((Instruction::Load(Load::LdRN(Register8::A)), 1)),

            0x70 => Ok((Instruction::Load(Load::LdRN(Register8::B)), 1)),
            0x71 => Ok((Instruction::Load(Load::LdRN(Register8::D)), 1)),
            0x72 => Ok((Instruction::Load(Load::LdRN(Register8::H)), 1)),
            0x73 => Ok((Instruction::Load(Load::LdRN(Register8::C)), 1)),
            0x74 => Ok((Instruction::Load(Load::LdRN(Register8::E)), 1)),
            0x75 => Ok((Instruction::Load(Load::LdRN(Register8::L)), 1)),
            0x77 => Ok((Instruction::Load(Load::LdRN(Register8::A)), 1)),

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

            0x80 => Ok((Instruction::Arith(Arith::AddR(Register8::B)), 1)),
            0x81 => Ok((Instruction::Arith(Arith::AddR(Register8::C)), 1)),
            0x82 => Ok((Instruction::Arith(Arith::AddR(Register8::D)), 1)),
            0x83 => Ok((Instruction::Arith(Arith::AddR(Register8::E)), 1)),
            0x84 => Ok((Instruction::Arith(Arith::AddR(Register8::H)), 1)),
            0x85 => Ok((Instruction::Arith(Arith::AddR(Register8::L)), 1)),
            0x86 => Ok((Instruction::Arith(Arith::AddN), 1)),
            0x87 => Ok((Instruction::Arith(Arith::AddR(Register8::A)), 1)),
            0xC6 => Ok((Instruction::Arith(Arith::AddI(bytes[1])), 2)),

            0x90 => Ok((Instruction::Arith(Arith::SubR(Register8::B)), 1)),
            0x91 => Ok((Instruction::Arith(Arith::SubR(Register8::C)), 1)),
            0x92 => Ok((Instruction::Arith(Arith::SubR(Register8::D)), 1)),
            0x93 => Ok((Instruction::Arith(Arith::SubR(Register8::E)), 1)),
            0x94 => Ok((Instruction::Arith(Arith::SubR(Register8::H)), 1)),
            0x95 => Ok((Instruction::Arith(Arith::SubR(Register8::L)), 1)),
            0x96 => Ok((Instruction::Arith(Arith::SubN), 1)),
            0x97 => Ok((Instruction::Arith(Arith::SubR(Register8::A)), 1)),
            0xD6 => Ok((Instruction::Arith(Arith::SubI(bytes[1])), 2)),

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

            0xA8 => Ok((Instruction::Logic(Logic::XorR(Register8::B)), 1)),
            0xA9 => Ok((Instruction::Logic(Logic::XorR(Register8::C)), 1)),
            0xAA => Ok((Instruction::Logic(Logic::XorR(Register8::D)), 1)),
            0xAB => Ok((Instruction::Logic(Logic::XorR(Register8::E)), 1)),
            0xAC => Ok((Instruction::Logic(Logic::XorR(Register8::H)), 1)),
            0xAD => Ok((Instruction::Logic(Logic::XorR(Register8::L)), 1)),
            0xAF => Ok((Instruction::Logic(Logic::XorR(Register8::A)), 1)),

            0xC9 => Ok((Instruction::Control(Control::Ret), 1)),
            0xD9 => Ok((Instruction::Control(Control::Ret), 1)),

            0xC0 => Ok((Instruction::Control(Control::RetNZ), 1)),
            0xD0 => Ok((Instruction::Control(Control::RetNC), 1)),
            0xC8 => Ok((Instruction::Control(Control::RetZ), 1)),
            0xD8 => Ok((Instruction::Control(Control::RetC), 1)),

            0x17 => Ok((Instruction::Bits(Bits::Rla), 1)),
            0x1F => Ok((Instruction::Bits(Bits::Rra), 1)),
            0x07 => Ok((Instruction::Bits(Bits::Rlca), 1)),
            0x0F => Ok((Instruction::Bits(Bits::Rrca), 1)),

            0xCB => match bytes[1] {
                0x30 => Ok((Instruction::Bits(Bits::SwapR(Register8::B)), 1)),
                0x31 => Ok((Instruction::Bits(Bits::SwapR(Register8::C)), 1)),
                0x32 => Ok((Instruction::Bits(Bits::SwapR(Register8::D)), 1)),
                0x33 => Ok((Instruction::Bits(Bits::SwapR(Register8::E)), 1)),
                0x34 => Ok((Instruction::Bits(Bits::SwapR(Register8::H)), 1)),
                0x35 => Ok((Instruction::Bits(Bits::SwapR(Register8::L)), 1)),
                0x37 => Ok((Instruction::Bits(Bits::SwapR(Register8::A)), 1)),

                0x20 => Ok((Instruction::Bits(Bits::SlaR(Register8::B)), 1)),
                0x21 => Ok((Instruction::Bits(Bits::SlaR(Register8::C)), 1)),
                0x22 => Ok((Instruction::Bits(Bits::SlaR(Register8::D)), 1)),
                0x23 => Ok((Instruction::Bits(Bits::SlaR(Register8::E)), 1)),
                0x24 => Ok((Instruction::Bits(Bits::SlaR(Register8::H)), 1)),
                0x25 => Ok((Instruction::Bits(Bits::SlaR(Register8::L)), 1)),
                0x27 => Ok((Instruction::Bits(Bits::SlaR(Register8::A)), 1)),

                0x10 => Ok((Instruction::Bits(Bits::RlR(Register8::B)), 1)),
                0x11 => Ok((Instruction::Bits(Bits::RlR(Register8::C)), 1)),
                0x12 => Ok((Instruction::Bits(Bits::RlR(Register8::D)), 1)),
                0x13 => Ok((Instruction::Bits(Bits::RlR(Register8::E)), 1)),
                0x14 => Ok((Instruction::Bits(Bits::RlR(Register8::H)), 1)),
                0x15 => Ok((Instruction::Bits(Bits::RlR(Register8::L)), 1)),
                0x17 => Ok((Instruction::Bits(Bits::RlR(Register8::A)), 1)),

                0x80 | 0x88 | 0x90 | 0x98 | 0xA0 | 0xA8 | 0xB0 | 0xB8 => Ok((
                    Instruction::Bits(Bits::Res(get_bits_bit(bytes[1]), Register8::B)),
                    2,
                )),
                0x81 | 0x89 | 0x91 | 0x99 | 0xA1 | 0xA9 | 0xB1 | 0xB9 => Ok((
                    Instruction::Bits(Bits::Res(get_bits_bit(bytes[1]), Register8::C)),
                    2,
                )),
                0x82 | 0x8A | 0x92 | 0x9A | 0xA2 | 0xAA | 0xB2 | 0xBA => Ok((
                    Instruction::Bits(Bits::Res(get_bits_bit(bytes[1]), Register8::D)),
                    2,
                )),
                0x83 | 0x8B | 0x93 | 0x9B | 0xA3 | 0xAB | 0xB3 | 0xBB => Ok((
                    Instruction::Bits(Bits::Res(get_bits_bit(bytes[1]), Register8::E)),
                    2,
                )),
                0x84 | 0x8C | 0x94 | 0x9C | 0xA4 | 0xAC | 0xB4 | 0xBC => Ok((
                    Instruction::Bits(Bits::Res(get_bits_bit(bytes[1]), Register8::H)),
                    2,
                )),
                0x85 | 0x8D | 0x95 | 0x9D | 0xA5 | 0xAD | 0xB5 | 0xBD => Ok((
                    Instruction::Bits(Bits::Res(get_bits_bit(bytes[1]), Register8::L)),
                    2,
                )),
                0x87 | 0x8F | 0x97 | 0x9F | 0xA7 | 0xAF | 0xB6 | 0xBF => Ok((
                    Instruction::Bits(Bits::Res(get_bits_bit(bytes[1]), Register8::A)),
                    2,
                )),

                0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8 => Ok((
                    Instruction::Bits(Bits::Set(get_bits_bit(bytes[1]), Register8::B)),
                    2,
                )),
                0xC1 | 0xC9 | 0xD1 | 0xD9 | 0xE1 | 0xE9 | 0xF1 | 0xF9 => Ok((
                    Instruction::Bits(Bits::Set(get_bits_bit(bytes[1]), Register8::C)),
                    2,
                )),
                0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA => Ok((
                    Instruction::Bits(Bits::Set(get_bits_bit(bytes[1]), Register8::D)),
                    2,
                )),
                0xC3 | 0xCB | 0xD3 | 0xDB | 0xE3 | 0xEB | 0xF3 | 0xFB => Ok((
                    Instruction::Bits(Bits::Set(get_bits_bit(bytes[1]), Register8::E)),
                    2,
                )),
                0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => Ok((
                    Instruction::Bits(Bits::Set(get_bits_bit(bytes[1]), Register8::H)),
                    2,
                )),
                0xC5 | 0xCD | 0xD5 | 0xDD | 0xE5 | 0xED | 0xF5 | 0xFD => Ok((
                    Instruction::Bits(Bits::Set(get_bits_bit(bytes[1]), Register8::L)),
                    2,
                )),
                0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF6 | 0xFF => Ok((
                    Instruction::Bits(Bits::Set(get_bits_bit(bytes[1]), Register8::A)),
                    2,
                )),

                _ => {
                    error!(
                        "Unknown instruction {:#X} {:#X} {:#X}",
                        bytes[0], bytes[1], bytes[2]
                    );
                    Err(())
                }
            },
            _ => {
                error!(
                    "Unknown instruction {:#X} {:#X} {:#X}",
                    bytes[0], bytes[1], bytes[2]
                );
                Err(())
            }
        }
    }
}

fn get_bits_bit(i: u8) -> u8 {
    (i >> 3) & 0b111
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::Nop => write!(f, "nop"),
            Instruction::Ei => write!(f, "ei"),
            Instruction::Di => write!(f, "di"),
            Instruction::Halt => write!(f, "halt"),
            Instruction::Scf => write!(f, "scf"),
            Instruction::CpI(b) => write!(f, "cp {:#x}", b),
            Instruction::CpR(r) => write!(f, "cp {}", r),
            Instruction::Arith(a) => a.fmt(f),
            Instruction::Bits(b) => b.fmt(f),
            Instruction::Load(l) => l.fmt(f),
            Instruction::Control(c) => c.fmt(f),
            Instruction::Logic(l) => l.fmt(f),
        }
    }
}
