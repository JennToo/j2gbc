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

    f.set_subtract(true);
    f.set_carry(l < r);
    f.set_halfcarry(l > r);
    f.set_zero(l == r);

    (v, f)
}

pub fn add(l: u8, r: u8, mut f: Flags) -> (u8, Flags) {
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
    f.set_carry((l as u32) + (r as u32) > 0xFF);
    (v, f)
}

pub fn inc(l: u8, mut f: Flags) -> (u8, Flags) {
    let v = (Wrapping(l) + Wrapping(1)).0;
    f.set_zero(v == 0);
    f.set_halfcarry(l == 0xFF);
    f.set_subtract(false);

    (v, f)
}

pub fn dec(l: u8, mut f: Flags) -> (u8, Flags) {
    let v = (Wrapping(l) - Wrapping(1)).0;
    f.set_zero(v == 0);
    f.set_halfcarry(l == 0);
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
