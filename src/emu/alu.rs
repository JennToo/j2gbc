use std::num::Wrapping;

// TODO: Audit the setting of flags

pub const MASK_FLAG_Z: u8 = 0b1000_0000;
pub const MASK_FLAG_N: u8 = 0b0100_0000;
pub const MASK_FLAG_H: u8 = 0b0010_0000;
pub const MASK_FLAG_C: u8 = 0b0001_0000;

pub struct Flags(pub u8);

impl Flags {
    pub fn set_zero(&mut self, v: bool) {
        if v {
            self.0 |= MASK_FLAG_Z;
        } else {
            self.0 &= !MASK_FLAG_Z;
        }
    }

    pub fn set_subtract(&mut self, v: bool) {
        if v {
            self.0 |= MASK_FLAG_N;
        } else {
            self.0 &= !MASK_FLAG_N;
        }
    }

    pub fn set_carry(&mut self, v: bool) {
        if v {
            self.0 |= MASK_FLAG_C;
        } else {
            self.0 &= !MASK_FLAG_C;
        }
    }

    pub fn set_halfcarry(&mut self, v: bool) {
        if v {
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
}

pub fn hi_lo(hi: u8, lo: u8) -> u16 {
    (hi as u16) << 8 | lo as u16
}

pub fn hi(v: u16) -> u8 {
    ((v >> 8) & 0xFF) as u8
}

pub fn lo(v: u16) -> u8 {
    (v & 0xFF) as u8
}

pub fn sub(l: u8, r: u8) -> (u8, Flags) {
    let mut f = Flags(0);
    let v = (Wrapping(l) - Wrapping(r)).0;
    f.set_zero(v == 0);
    f.set_subtract(true);
    f.set_carry((l as i8) < (r as i8));
    f.set_halfcarry((l as i8) & 0x0F < (r as i8) & 0x0F);
    (v, f)
}

pub fn add(l: u8, r: u8) -> (u8, Flags) {
    let mut f = Flags(0);
    let v = (Wrapping(l) + Wrapping(r)).0;
    f.set_zero(v == 0);
    f.set_halfcarry((l & 0x0F) + (r & 0x0F) > 0x0F);
    f.set_carry((l as u16) + (r as u16) > 0xFF);
    f.set_subtract(false);
    (v, f)
}

pub fn add16(l: u16, r: u16, mut f: Flags) -> (u16, Flags) {
    let v = ((Wrapping(l) + Wrapping(r))).0;
    f.set_subtract(false);
    f.set_halfcarry((l & 0x0FFF) + (r & 0x0FFF) > 0x0FFF);
    f.set_carry((l as u32) + (r as u32) > 0xFFFF);
    (v, f)
}

pub fn inc(l: u8, mut f: Flags) -> (u8, Flags) {
    let v = (Wrapping(l) + Wrapping(1)).0;
    f.set_zero(v == 0);
    f.set_halfcarry((l & 0x0F) + 1 == 0x10);
    f.set_subtract(false);

    (v, f)
}

pub fn dec(l: u8, mut f: Flags) -> (u8, Flags) {
    let v = (Wrapping(l) - Wrapping(1)).0;
    f.set_zero(v == 0);
    f.set_halfcarry((l as i8) & 0x0F < (1 as i8) & 0x0F);
    f.set_subtract(true);

    (v, f)
}

pub fn and(l: u8, r: u8) -> (u8, Flags) {
    let mut f = Flags(0);
    let v = l & r;
    f.set_halfcarry(true);
    f.set_zero(v == 0);
    (v, f)
}

pub fn or(l: u8, r: u8) -> (u8, Flags) {
    let mut f = Flags(0);
    let v = l | r;
    f.set_zero(v == 0);
    (v, f)
}

pub fn xor(l: u8, r: u8) -> (u8, Flags) {
    let mut f = Flags(0);
    let v = l ^ r;
    f.set_zero(v == 0);
    (v, f)
}

pub fn swap(v: u8) -> (u8, Flags) {
    let mut f = Flags(0);
    f.set_zero(v == 0);
    (v << 4 | v >> 4, f)
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
