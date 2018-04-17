use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::convert::Into;
use std::fmt::{Debug, Formatter, Result};
use cart::Cart;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Address(pub u16);

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Address({:#X})", self.0)
    }
}

impl Add<Address> for Address {
    type Output = Address;

    fn add(self, o: Address) -> Address {
        Address(self.0 + o.0)
    }
}

impl AddAssign<Address> for Address {
    fn add_assign(&mut self, o: Address) {
        self.0 += o.0;
    }
}

impl Sub<Address> for Address {
    type Output = Address;

    fn sub(self, o: Address) -> Address {
        Address(self.0 - o.0)
    }
}

impl SubAssign<Address> for Address {
    fn sub_assign(&mut self, o: Address) {
        self.0 -= o.0;
    }
}

impl Into<usize> for Address {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Into<u16> for Address {
    fn into(self) -> u16 {
        self.0 as u16
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct AddressRange(pub Address, pub Address);

impl Address {
    pub fn in_(self, a: AddressRange) -> bool {
        self >= a.0 && self < a.1
    }
}

impl AddressRange {
    pub fn len(self) -> usize {
        (self.1 - self.0).into()
    }
}

pub trait MemDevice {
    fn read(&self, a: Address) -> u8;
    fn write(&mut self, a: Address, v: u8);

    fn write16(&mut self, a: Address, v: u16) {
        self.write(a, ((v >> 8) & 0xFF) as u8);
        self.write(a + Address(1), (v & 0xFF) as u8);
    }
}

pub struct Ram {
    data: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Ram {
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(0);
        }
        Ram { data }
    }
}

impl MemDevice for Ram {
    fn read(&self, a: Address) -> u8 {
        self.data[a.0 as usize]
    }

    fn write(&mut self, a: Address, v: u8) {
        self.data[a.0 as usize] = v;
    }
}

pub struct Mmu {
    internal_ram: Ram,
    tiny_ram: Ram,
    cart: Cart,
    // TODO: Actually implement IE register
    interrupt_enable: u8,
}

const RNG_INT_RAM: AddressRange = AddressRange(Address(0xC000), Address(0xE000));
const RNG_INT_TINY_RAM: AddressRange = AddressRange(Address(0xFF80), Address(0xFFFF));
pub const RNG_ROM_BANK0: AddressRange = AddressRange(Address(0x0000), Address(0x4000));
const OFF_INTR_ENABLE_REG: Address = Address(0xFFFF);

impl Mmu {
    pub fn new(cart: Cart) -> Mmu {
        Mmu {
            internal_ram: Ram::new(RNG_INT_RAM.len()),
            tiny_ram: Ram::new(RNG_INT_TINY_RAM.len()),
            interrupt_enable: 0,
            cart,
        }
    }
}

impl MemDevice for Mmu {
    fn read(&self, a: Address) -> u8 {
        if a.in_(RNG_INT_RAM) {
            self.internal_ram.read(a - RNG_INT_RAM.0)
        } else if a.in_(RNG_ROM_BANK0) {
            self.cart.read(a)
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.read(a - RNG_INT_TINY_RAM.0)
        } else if a == OFF_INTR_ENABLE_REG {
            self.interrupt_enable
        } else {
            panic!("MMU: Unimplemented memory read at address {:?}", a);
        }
    }

    fn write(&mut self, a: Address, v: u8) {
        if a.in_(RNG_INT_RAM) {
            self.internal_ram.write(a - RNG_INT_RAM.0, v);
        } else if a.in_(RNG_ROM_BANK0) {
            self.cart.write(a, v);
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.write(a - RNG_INT_TINY_RAM.0, v);
        } else if a == OFF_INTR_ENABLE_REG {
            self.interrupt_enable = v;
        } else {
            panic!("MMU: Unimplemented memory write at address {:?}", a);
        }
    }
}
