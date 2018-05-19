use std::fmt;
use std::fmt::Display;

use super::alu::hi_lo;
use super::cpu::{Operand, Register16, Register8};
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
    Ccf,
    Cp(Operand),
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
            Instruction::Scf | Instruction::Ccf => 4,
            Instruction::Cp(Operand::Immediate(_)) => 8,
            Instruction::Cp(Operand::IndirectRegister(_)) => 8,
            Instruction::Cp(Operand::Register(_)) => 4,

            Instruction::Arith(a) => a.cycles(),
            Instruction::Bits(b) => b.cycles(),
            Instruction::Load(l) => l.cycles(),
            Instruction::Control(c) => c.cycles(),
            Instruction::Logic(l) => l.cycles(),

            Instruction::Cp(_) => unimplemented!(),
        }
    }

    pub fn decode(bytes: [u8; 3]) -> Result<(Instruction, u8), ()> {
        match bytes[0] {
            0 => Ok((Instruction::Nop, 1)),

            0xFB => Ok((Instruction::Ei, 1)),
            0xF3 => Ok((Instruction::Di, 1)),

            0x76 => Ok((Instruction::Halt, 1)),

            0x37 => Ok((Instruction::Scf, 1)),
            0x3F => Ok((Instruction::Ccf, 1)),

            0x04 | 0x14 | 0x24 | 0x34 | 0x0C | 0x1C | 0x2C | 0x3C => Ok((
                Instruction::Arith(Arith::Inc(Operand::from_bits(bytes[0], 3))),
                1,
            )),

            0x05 | 0x15 | 0x25 | 0x35 | 0x0D | 0x1D | 0x2D | 0x3D => Ok((
                Instruction::Arith(Arith::Dec(Operand::from_bits(bytes[0], 3))),
                1,
            )),

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
            0xC4 => Ok((
                Instruction::Control(Control::CallINZ(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xD4 => Ok((
                Instruction::Control(Control::CallINC(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xCC => Ok((
                Instruction::Control(Control::CallIZ(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),
            0xDC => Ok((
                Instruction::Control(Control::CallIC(Address(hi_lo(bytes[2], bytes[1])))),
                3,
            )),

            0xF0 => Ok((
                Instruction::Load(Load::Ld(
                    Operand::Register(Register8::A),
                    Operand::IndirectAddress(Address(0xFF00) + Address(u16::from(bytes[1]))),
                )),
                2,
            )),
            0xE0 => Ok((
                Instruction::Load(Load::Ld(
                    Operand::IndirectAddress(Address(0xFF00) + Address(u16::from(bytes[1]))),
                    Operand::Register(Register8::A),
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

            0xF8 => Ok((Instruction::Load(Load::LdHLSPI(bytes[1] as i8)), 2)),
            0xF9 => Ok((Instruction::Load(Load::LdSPHL), 1)),

            0x02 => Ok((Instruction::Load(Load::LdNR16(Register16::BC)), 1)),
            0x12 => Ok((Instruction::Load(Load::LdNR16(Register16::DE)), 1)),

            0x0A => Ok((Instruction::Load(Load::LdRN16(Register16::BC)), 1)),
            0x1A => Ok((Instruction::Load(Load::LdRN16(Register16::DE)), 1)),

            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => Ok((
                Instruction::Load(Load::Ld(
                    Operand::from_bits(bytes[0], 3),
                    Operand::Immediate(bytes[1]),
                )),
                2,
            )),

            0x40...0x75 | 0x77...0x7F => Ok((
                Instruction::Load(Load::Ld(
                    Operand::from_bits(bytes[0], 3),
                    Operand::from_bits(bytes[0], 0),
                )),
                1,
            )),

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

            0xFE => Ok((Instruction::Cp(Operand::Immediate(bytes[1])), 2)),

            0xB8...0xBF => Ok((Instruction::Cp(Operand::from_bits(bytes[0], 0)), 1)),

            0x20 => Ok((Instruction::Control(Control::JrNZI(bytes[1] as i8)), 2)),
            0x30 => Ok((Instruction::Control(Control::JrNCI(bytes[1] as i8)), 2)),
            0x18 => Ok((Instruction::Control(Control::JrI(bytes[1] as i8)), 2)),
            0x28 => Ok((Instruction::Control(Control::JrZI(bytes[1] as i8)), 2)),
            0x38 => Ok((Instruction::Control(Control::JrCI(bytes[1] as i8)), 2)),

            0x80...0x87 => Ok((
                Instruction::Arith(Arith::Add(Operand::from_bits(bytes[0], 0))),
                1,
            )),
            0xC6 => Ok((
                Instruction::Arith(Arith::Add(Operand::Immediate(bytes[1]))),
                2,
            )),

            0x88...0x8F => Ok((
                Instruction::Arith(Arith::Adc(Operand::from_bits(bytes[0], 0))),
                1,
            )),
            0xCE => Ok((
                Instruction::Arith(Arith::Adc(Operand::Immediate(bytes[1]))),
                2,
            )),

            0x90...0x97 => Ok((
                Instruction::Arith(Arith::Sub(Operand::from_bits(bytes[0], 0))),
                1,
            )),
            0xD6 => Ok((
                Instruction::Arith(Arith::Sub(Operand::Immediate(bytes[1]))),
                2,
            )),

            0x98...0x9F => Ok((
                Instruction::Arith(Arith::Sbc(Operand::from_bits(bytes[0], 0))),
                1,
            )),
            0xDE => Ok((
                Instruction::Arith(Arith::Sbc(Operand::Immediate(bytes[1]))),
                2,
            )),

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
            0xB7 => Ok((Instruction::Logic(Logic::OrR(Register8::A)), 1)),

            0xEE => Ok((Instruction::Logic(Logic::XorI(bytes[1])), 2)),
            0xAE => Ok((Instruction::Logic(Logic::XorN), 1)),

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
                0x10...0x17 => Ok((
                    Instruction::Bits(Bits::Rl(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x18...0x1F => Ok((
                    Instruction::Bits(Bits::Rr(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x20...0x27 => Ok((
                    Instruction::Bits(Bits::Sla(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x30...0x37 => Ok((
                    Instruction::Bits(Bits::Swap(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x38...0x3F => Ok((
                    Instruction::Bits(Bits::Srl(Operand::from_bits(bytes[1], 0))),
                    2,
                )),
                0x40...0x7F => Ok((
                    Instruction::Bits(Bits::Bit(
                        get_bits_bit(bytes[1]),
                        Operand::from_bits(bytes[1], 0),
                    )),
                    2,
                )),
                0x80...0xBF => Ok((
                    Instruction::Bits(Bits::Res(
                        get_bits_bit(bytes[1]),
                        Operand::from_bits(bytes[1], 0),
                    )),
                    2,
                )),
                0xC0...0xFF => Ok((
                    Instruction::Bits(Bits::Res(
                        get_bits_bit(bytes[1]),
                        Operand::from_bits(bytes[1], 0),
                    )),
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
            Instruction::Ccf => write!(f, "ccf"),
            Instruction::Cp(o) => write!(f, "cp {}", o),
            Instruction::Arith(a) => a.fmt(f),
            Instruction::Bits(b) => b.fmt(f),
            Instruction::Load(l) => l.fmt(f),
            Instruction::Control(c) => c.fmt(f),
            Instruction::Logic(l) => l.fmt(f),
        }
    }
}
