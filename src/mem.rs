use std::ops::Add;
use cart::Cart;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct Address(pub u16);

pub trait MemDevice {
    fn read(&self, a: Address) -> u8;
    fn write(&mut self, a: Address, v: u8);

    fn write16(&mut self, a: Address, v: u16) {
        self.write(a, ((v >> 8) & 0xFF) as u8);
        self.write(Address(a.0 + 1), (v & 0xFF) as u8);
    }
}

pub struct Ram {
    data: Vec<u8>,
    location: Address,
}

impl Ram {
    pub fn new(size: usize, location: Address) -> Ram {
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(0);
        }
        Ram {
            data,
            location,
        }
    }
}

impl MemDevice for Ram {
    fn read(&self, a: Address) -> u8 {
        self.data[(a.0 - self.location.0) as usize]
    }

    fn write(&mut self, a: Address, v: u8) {
        self.data[(a.0 - self.location.0) as usize] = v;
    }
}

pub struct Mmu {
    internal_ram: Ram,
    tiny_ram: Ram,
    cart: Cart,
    interrupt_enable: u8,
}

const OFF_INT_RAM_START: Address = Address(0xC000);
const OFF_INT_RAM_END: Address = Address(0xE000);
const OFF_INT_TINY_RAM_START: Address = Address(0xFF80);
const OFF_INT_TINY_RAM_END: Address = Address(0xFFFF);
pub const OFF_ROM_BANK0_START: Address = Address(0x0000);
pub const OFF_ROM_BANK0_END: Address = Address(0x4000);
const OFF_INTR_ENABLE_REG: Address = Address(0xFFFF);

impl Mmu {
    pub fn new(cart: Cart) -> Mmu {
        Mmu {
            internal_ram: Ram::new((OFF_INT_RAM_END.0 - OFF_INT_RAM_START.0) as usize, OFF_INT_RAM_START),
            tiny_ram: Ram::new((OFF_INT_TINY_RAM_END.0 - OFF_INT_TINY_RAM_START.0) as usize, OFF_INT_TINY_RAM_START),
            interrupt_enable: 0,
            cart,
        }
    }
}

impl MemDevice for Mmu {
    fn read(&self, a: Address) -> u8 {
        if a >= OFF_INT_RAM_START && a < OFF_INT_RAM_END {
            self.internal_ram.read(a)
        } else if a >= OFF_ROM_BANK0_START && a < OFF_ROM_BANK0_END {
            self.cart.read(a)
        } else if a >= OFF_INT_TINY_RAM_START && a < OFF_INT_TINY_RAM_END {
            self.tiny_ram.read(a)
        } else if a == OFF_INTR_ENABLE_REG {
            self.interrupt_enable
        } else {
            panic!("MMU: Unimplemented memory read at address {:?}", a);
        }
    }

    fn write(&mut self, a: Address, v: u8) {
        if a >= OFF_INT_RAM_START && a < OFF_INT_RAM_END {
            self.internal_ram.write(a, v);
        } else if a >= OFF_ROM_BANK0_START && a < OFF_ROM_BANK0_END {
            self.cart.write(a, v);
        } else if a >= OFF_INT_TINY_RAM_START && a < OFF_INT_TINY_RAM_END {
            self.tiny_ram.write(a, v);
        } else if a == OFF_INTR_ENABLE_REG {
            self.interrupt_enable = v;
        } else {
            panic!("MMU: Unimplemented memory write at address {:?}", a);
        }
    }
}
