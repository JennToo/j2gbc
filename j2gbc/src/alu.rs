use std::num::Wrapping;

pub const MASK_FLAG_Z: u8 = 0b1000_0000;
pub const MASK_FLAG_N: u8 = 0b0100_0000;
pub const MASK_FLAG_H: u8 = 0b0010_0000;
pub const MASK_FLAG_C: u8 = 0b0001_0000;

use crate::cpu::ConditionCode;

pub struct Flags(pub u8);

impl Flags {
    #[cfg(test)]
    pub fn carry(mut self) -> Flags {
        self.set_carry(true);
        self
    }

    #[cfg(test)]
    pub fn halfcarry(mut self) -> Flags {
        self.set_halfcarry(true);
        self
    }

    #[cfg(test)]
    pub fn subtract(mut self) -> Flags {
        self.set_subtract(true);
        self
    }

    #[cfg(test)]
    pub fn zero(mut self) -> Flags {
        self.set_zero(true);
        self
    }

    pub fn set_zero(&mut self, value: bool) {
        if value {
            self.0 |= MASK_FLAG_Z;
        } else {
            self.0 &= !MASK_FLAG_Z;
        }
    }

    pub fn set_subtract(&mut self, value: bool) {
        if value {
            self.0 |= MASK_FLAG_N;
        } else {
            self.0 &= !MASK_FLAG_N;
        }
    }

    pub fn set_carry(&mut self, value: bool) {
        if value {
            self.0 |= MASK_FLAG_C;
        } else {
            self.0 &= !MASK_FLAG_C;
        }
    }

    pub fn set_halfcarry(&mut self, value: bool) {
        if value {
            self.0 |= MASK_FLAG_H;
        } else {
            self.0 &= !MASK_FLAG_H;
        }
    }

    pub fn get_zero(&self) -> bool {
        self.0 & MASK_FLAG_Z != 0
    }

    pub fn get_subtract(&self) -> bool {
        self.0 & MASK_FLAG_N != 0
    }

    pub fn get_carry(&self) -> bool {
        self.0 & MASK_FLAG_C != 0
    }

    pub fn get_halfcarry(&self) -> bool {
        self.0 & MASK_FLAG_H != 0
    }

    pub fn matches(self, cond: ConditionCode) -> bool {
        match cond {
            ConditionCode::NotZero => !self.get_zero(),
            ConditionCode::NotCarry => !self.get_carry(),
            ConditionCode::Zero => self.get_zero(),
            ConditionCode::Carry => self.get_carry(),
        }
    }
}

pub fn hi_lo(hi: u8, lo: u8) -> u16 {
    u16::from(hi) << 8 | u16::from(lo)
}

pub fn hi(value: u16) -> u8 {
    ((value >> 8) & 0xFF) as u8
}

pub fn lo(value: u16) -> u8 {
    (value & 0xFF) as u8
}

pub fn sub(left: u8, right: u8) -> (u8, Flags) {
    let mut flags = Flags(0);
    let result = (Wrapping(left) - Wrapping(right)).0;
    flags.set_zero(result == 0);
    flags.set_subtract(true);
    flags.set_carry(left < right);
    flags.set_halfcarry(left & 0x0F < right & 0x0F);
    (result, flags)
}

pub fn sbc(left: u8, right: u8, carry: bool) -> (u8, Flags) {
    let mut flags = Flags(0);
    let carry_as_value = if carry { 1 } else { 0 };
    let result = (Wrapping(left) - Wrapping(right) - Wrapping(carry_as_value)).0;
    let full_o = (Wrapping(left) - Wrapping(right)).0;
    flags.set_zero(result == 0);
    flags.set_subtract(true);
    flags.set_carry(left < right || full_o < carry_as_value);
    flags.set_halfcarry(left & 0x0F < right & 0x0F || full_o & 0x0F < carry_as_value);
    (result, flags)
}

pub fn add(left: u8, right: u8) -> (u8, Flags) {
    let mut flags = Flags(0);
    let result = (Wrapping(left) + Wrapping(right)).0;
    flags.set_zero(result == 0);
    flags.set_halfcarry((left & 0x0F) + (right & 0x0F) > 0x0F);
    flags.set_carry(u16::from(left) + u16::from(right) > 0xFF);
    flags.set_subtract(false);
    (result, flags)
}

pub fn adc(left: u8, right: u8, carry: bool) -> (u8, Flags) {
    let mut flags = Flags(0);
    let carry_as_value = if carry { 1 } else { 0 };
    let result = (Wrapping(left) + Wrapping(right) + Wrapping(carry_as_value)).0;
    let full_l = u16::from(left) + u16::from(carry_as_value);
    flags.set_zero(result == 0);
    flags.set_halfcarry(
        (full_l & 0x0F) as u8 + (right & 0x0F) > 0x0F
            || (left & 0x0F == 0x0F && carry_as_value == 1),
    );
    flags.set_carry(full_l + u16::from(right) > 0xFF);
    flags.set_subtract(false);
    (result, flags)
}

pub fn add16(left: u16, right: u16, mut flags: Flags) -> (u16, Flags) {
    let result = (Wrapping(left) + Wrapping(right)).0;
    flags.set_subtract(false);
    flags.set_halfcarry((left & 0x0FFF) + (right & 0x0FFF) > 0x0FFF);
    flags.set_carry(u32::from(left) + u32::from(right) > 0xFFFF);
    (result, flags)
}

pub fn inc(value: u8, mut flags: Flags) -> (u8, Flags) {
    let result = (Wrapping(value) + Wrapping(1)).0;
    flags.set_zero(result == 0);
    flags.set_halfcarry((value & 0x0F) + 1 == 0x10);
    flags.set_subtract(false);

    (result, flags)
}

pub fn dec(value: u8, mut flags: Flags) -> (u8, Flags) {
    let result = (Wrapping(value) - Wrapping(1)).0;
    flags.set_zero(result == 0);
    flags.set_halfcarry((value as i8) & 0x0F < (1 as i8) & 0x0F);
    flags.set_subtract(true);

    (result, flags)
}

pub fn and(left: u8, right: u8) -> (u8, Flags) {
    let mut flags = Flags(0);
    let result = left & right;
    flags.set_halfcarry(true);
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn or(left: u8, right: u8) -> (u8, Flags) {
    let mut flags = Flags(0);
    let result = left | right;
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn xor(left: u8, right: u8) -> (u8, Flags) {
    let mut flags = Flags(0);
    let result = left ^ right;
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn swap(value: u8) -> (u8, Flags) {
    let mut flags = Flags(0);
    flags.set_zero(value == 0);
    (value << 4 | value >> 4, flags)
}

pub fn sla(value: u8) -> (u8, Flags) {
    let mut flags = Flags(0);
    let result = value << 1;
    flags.set_carry(value & 0b1000_0000 != 0);
    flags.set_halfcarry(false);
    flags.set_subtract(false);
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn sra(value: u8) -> (u8, Flags) {
    let mut flags = Flags(0);
    let result = value >> 1 | (0b1000_0000 & value);
    flags.set_carry(value & 0b1 != 0);
    flags.set_halfcarry(false);
    flags.set_subtract(false);
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn srl(value: u8) -> (u8, Flags) {
    let mut flags = Flags(0);
    let result = value >> 1;
    flags.set_carry(value & 0b0000_0001 != 0);
    flags.set_halfcarry(false);
    flags.set_subtract(false);
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn rl(value: u8, mut flags: Flags) -> (u8, Flags) {
    let mut result = value << 1;
    if flags.get_carry() {
        result |= 1;
    }
    flags.set_carry(value & 0b1000_0000 != 0);
    flags.set_halfcarry(false);
    flags.set_subtract(false);
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn rlc(value: u8, mut flags: Flags) -> (u8, Flags) {
    let result = value.rotate_left(1);
    flags.set_carry(value & 0b1000_0000 != 0);
    flags.set_halfcarry(false);
    flags.set_subtract(false);
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn rr(value: u8, mut flags: Flags) -> (u8, Flags) {
    let mut result = value >> 1;
    if flags.get_carry() {
        result |= 0b1000_0000;
    }
    flags.set_carry(value & 1 != 0);
    flags.set_halfcarry(false);
    flags.set_subtract(false);
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn rrc(value: u8, mut flags: Flags) -> (u8, Flags) {
    let result = value.rotate_right(1);
    flags.set_carry(value & 1 != 0);
    flags.set_halfcarry(false);
    flags.set_subtract(false);
    flags.set_zero(result == 0);
    (result, flags)
}

pub fn daa(value: u8, mut flags: Flags) -> (u8, Flags) {
    let mut value = Wrapping(value);
    let mut correction = 0;

    if flags.get_halfcarry() || (!flags.get_subtract() && value.0 & 0x0F > 0x09) {
        correction |= 0x06;
    }

    if flags.get_carry() || (!flags.get_subtract() && value.0 > 0x99) {
        correction |= 0x60;
        flags.set_carry(true);
    }

    if flags.get_subtract() {
        value -= Wrapping(correction);
    } else {
        value += Wrapping(correction);
    }

    flags.set_zero(value.0 == 0);
    flags.set_halfcarry(false);

    (value.0, flags)
}

#[test]
fn test_add() {
    let (v, f) = add(0x3A, 0xC6);
    assert_eq!(v, 0);
    assert!(f.get_zero());
    assert!(f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = add(0x3C, 0xFF);
    assert_eq!(v, 0x3B);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = add(0x3C, 0x12);
    assert_eq!(v, 0x4E);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = add(0xAF, 0xA1);
    assert_eq!(v, 0x50);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());
}

#[test]
fn test_sub() {
    let (v, f) = sub(0x3E, 0x3E);
    assert_eq!(v, 0);
    assert!(f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(f.get_subtract());

    let (v, f) = sub(0x3E, 0x0F);
    assert_eq!(v, 0x2F);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(f.get_subtract());

    let (v, f) = sub(0x3E, 0x40);
    assert_eq!(v, 0xFE);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(f.get_subtract());

    // Examples for CP which is the same as SUB essentially
    let (_, f) = sub(0x3C, 0x2F);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(f.get_subtract());

    let (_, f) = sub(0x3C, 0x3C);
    assert!(f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(f.get_subtract());

    let (_, f) = sub(0x3C, 0x40);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(f.get_subtract());
}

#[test]
fn test_adc() {
    let (v, f) = adc(0xE1, 0x0F, true);
    assert_eq!(v, 0xF1);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = adc(0xE1, 0x3B, true);
    assert_eq!(v, 0x1D);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = adc(0xE1, 0x1E, true);
    assert_eq!(v, 0x00);
    assert!(f.get_zero());
    assert!(f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());
}

#[test]
fn test_sbc() {
    let (v, f) = sbc(0x3B, 0x2A, true);
    assert_eq!(v, 0x10);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(f.get_subtract());

    let (v, f) = sbc(0x3B, 0x3A, true);
    assert_eq!(v, 0x0);
    assert!(f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(f.get_subtract());

    let (v, f) = sbc(0x3B, 0x4F, true);
    assert_eq!(v, 0xEB);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(f.get_carry());
    assert!(f.get_subtract());
}

#[test]
fn test_and() {
    let (v, f) = and(0x5A, 0x3F);
    assert_eq!(v, 0x1A);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(f.get_halfcarry());
    assert!(!f.get_zero());

    let (v, f) = and(0x5A, 0x38);
    assert_eq!(v, 0x18);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(f.get_halfcarry());
    assert!(!f.get_zero());

    let (v, f) = and(0x5A, 0x00);
    assert_eq!(v, 0x00);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(f.get_halfcarry());
    assert!(f.get_zero());
}

#[test]
fn test_or() {
    let (v, f) = or(0x5A, 0x5A);
    assert_eq!(v, 0x5A);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(!f.get_halfcarry());
    assert!(!f.get_zero());

    let (v, f) = or(0x00, 0x00);
    assert_eq!(v, 0x00);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(!f.get_halfcarry());
    assert!(f.get_zero());

    let (v, f) = or(0x5A, 0x03);
    assert_eq!(v, 0x5B);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(!f.get_halfcarry());
    assert!(!f.get_zero());

    let (v, f) = or(0x5A, 0x0);
    assert_eq!(v, 0x5A);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(!f.get_halfcarry());
    assert!(!f.get_zero());
}

#[test]
fn test_xor() {
    let (v, f) = xor(0xFF, 0xFF);
    assert_eq!(v, 0x00);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(!f.get_halfcarry());
    assert!(f.get_zero());

    let (v, f) = xor(0xFF, 0x0F);
    assert_eq!(v, 0xF0);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(!f.get_halfcarry());
    assert!(!f.get_zero());

    let (v, f) = xor(0xFF, 0x8A);
    assert_eq!(v, 0x75);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(!f.get_halfcarry());
    assert!(!f.get_zero());
}

#[test]
fn test_inc() {
    let (v, f) = inc(0xFF, Flags(0));
    assert_eq!(v, 0x00);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(f.get_halfcarry());
    assert!(f.get_zero());

    let (v, f) = inc(0x50, Flags(0));
    assert_eq!(v, 0x51);
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
    assert!(!f.get_halfcarry());
    assert!(!f.get_zero());
}

#[test]
fn test_dec() {
    let (v, f) = dec(0x01, Flags(0));
    assert_eq!(v, 0x00);
    assert!(!f.get_carry());
    assert!(f.get_subtract());
    assert!(!f.get_halfcarry());
    assert!(f.get_zero());

    let (v, f) = dec(0x00, Flags(0));
    assert_eq!(v, 0xFF);
    assert!(!f.get_carry());
    assert!(f.get_subtract());
    assert!(f.get_halfcarry());
    assert!(!f.get_zero());
}

#[test]
fn test_add16() {
    let (v, f) = add16(0x8A23, 0x0605, Flags(0));
    assert_eq!(v, 0x9028);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = add16(0x8A23, 0x8A23, Flags(0));
    assert_eq!(v, 0x1446);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = add16(0xFFFF, 0x0001, Flags(0));
    assert_eq!(v, 0);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());
}

#[test]
fn test_swap() {
    let (v, f) = swap(0x00);
    assert_eq!(v, 0x00);
    assert!(f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = swap(0xF0);
    assert_eq!(v, 0x0F);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
}

#[test]
fn test_sla() {
    let (v, f) = sla(0x80);
    assert_eq!(v, 0x00);
    assert!(f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = sla(0xFF);
    assert_eq!(v, 0xFE);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());
}

#[test]
fn test_sra() {
    let (v, f) = sra(0x8A);
    assert_eq!(v, 0xC5);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = sra(0x01);
    assert_eq!(v, 0x00);
    assert!(f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());
}

#[test]
fn test_srl() {
    let (v, f) = srl(0x01);
    assert_eq!(v, 0x00);
    assert!(f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = srl(0xFF);
    assert_eq!(v, 0x7F);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());
}

#[test]
fn test_rl() {
    let (v, f) = rl(0x80, Flags(0));
    assert_eq!(v, 0x00);
    assert!(f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = rl(0x11, Flags(0));
    assert_eq!(v, 0x22);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());

    let mut f = Flags(0);
    f.set_carry(true);
    let (v, f) = rl(0x00, f);
    assert_eq!(v, 0x01);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
}

#[test]
fn test_rr() {
    let (v, f) = rr(0x81, Flags(0));
    assert_eq!(v, 0x40);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = rr(0x01, Flags(0));
    assert_eq!(v, 0x00);
    assert!(f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = rr(0x8A, Flags(0));
    assert_eq!(v, 0x45);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());
}

#[test]
fn test_daa() {
    let (v, f) = add(0x45, 0x38);
    assert_eq!(v, 0x7D);
    let (v, f) = daa(v, f);
    assert_eq!(v, 0x83);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(!f.get_subtract());

    let (v, f) = sub(0x83, 0x38);
    assert_eq!(v, 0x4B);
    assert!(!f.get_zero());
    assert!(f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(f.get_subtract());
    let (v, f) = daa(v, f);
    assert_eq!(v, 0x45);
    assert!(!f.get_zero());
    assert!(!f.get_halfcarry());
    assert!(!f.get_carry());
    assert!(f.get_subtract());
}
