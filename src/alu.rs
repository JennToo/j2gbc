use std::num::Wrapping;

pub const MASK_FLAG_Z: u8 = 0b1000_0000;
pub const MASK_FLAG_N: u8 = 0b0100_0000;
pub const MASK_FLAG_H: u8 = 0b0010_0000;
pub const MASK_FLAG_C: u8 = 0b0001_0000;

pub fn hi_lo(hi: u8, lo: u8) -> u16 {
    (hi as u16) << 8 | lo as u16
}

pub fn hi(v: u16) -> u8 {
    ((v >> 8) & 0xFF) as u8
}

pub fn lo(v: u16) -> u8 {
    (v & 0xFF) as u8
}

pub fn sub(l: u8, r: u8) -> (u8, Wrapping<u8>) {
    if l < r {
        ((MASK_FLAG_N | MASK_FLAG_C), Wrapping(l) - Wrapping(r))
    } else if l > r {
        ((MASK_FLAG_N | MASK_FLAG_H), Wrapping(l) - Wrapping(r))
    } else {
        ((MASK_FLAG_N | MASK_FLAG_Z), Wrapping(l) - Wrapping(r))
    }
}

pub fn and(l: u8, r: u8) -> (u8, u8) {
    let v = l & r;
    if v == 0 {
        ((MASK_FLAG_H | MASK_FLAG_Z), v)
    } else {
        (MASK_FLAG_H, v)
    }
}

pub fn xor(l: u8, r: u8) -> (u8, u8) {
    let v = l ^ r;
    if v == 0 {
        (MASK_FLAG_Z, v)
    } else {
        (0, v)
    }
}
