use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::convert::Into;
use std::fmt::{Debug, Formatter};
use std::fmt;

use super::cart::Cart;
use super::lcd::Lcd;
use super::audio::Audio;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Address(pub u16);

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

impl Add<i8> for Address {
    type Output = Address;

    fn add(self, o: i8) -> Address {
        Address((self.0 as i32 + o as i32) as u16)
    }
}

impl AddAssign<i8> for Address {
    fn add_assign(&mut self, o: i8) {
        self.0 = (*self + o).0;
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
    fn read(&self, a: Address) -> Result<u8, ()>;
    fn write(&mut self, a: Address, v: u8) -> Result<(), ()>;

    fn write16(&mut self, a: Address, v: u16) -> Result<(), ()> {
        try!(self.write(a, ((v >> 8) & 0xFF) as u8));
        try!(self.write(a + Address(1), (v & 0xFF) as u8));
        Ok(())
    }

    fn read16(&self, a: Address) -> Result<u16, ()> {
        Ok((try!(self.read(a)) as u16) << 8 | (try!(self.read(a + Address(1))) as u16))
    }
}

#[derive(Clone, Debug)]
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
    fn read(&self, a: Address) -> Result<u8, ()> {
        Ok(self.data[a.0 as usize])
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        self.data[a.0 as usize] = v;
        Ok(())
    }
}

pub struct Mmu {
    internal_ram: Ram,
    tiny_ram: Ram,
    cart: Cart,
    // TODO: Actually implement IE register
    pub interrupt_enable: u8,
    pub interrupt_table: Ram,
    pub lcd: Box<Lcd>,
    audio: Audio,
}

pub const RNG_EXT_RAM: AddressRange = AddressRange(Address(0xA000), Address(0xC000));
const RNG_INT_RAM: AddressRange = AddressRange(Address(0xC000), Address(0xE000));
const RNG_INT_TINY_RAM: AddressRange = AddressRange(Address(0xFF80), Address(0xFFFF));
pub const RNG_INTR_TABLE: AddressRange = AddressRange(Address(0x0000), Address(0x0100));
pub const RNG_ROM_BANK0: AddressRange = AddressRange(Address(0x0100), Address(0x4000));
pub const RNG_ROM_BANK1: AddressRange = AddressRange(Address(0x4000), Address(0x8000));
const OFF_INTR_ENABLE_REG: Address = Address(0xFFFF);
const RNG_LCD_MM_REG: AddressRange = AddressRange(Address(0xFF40), Address(0xFF6C));
pub const RNG_CHAR_DAT: AddressRange = AddressRange(Address(0x8000), Address(0x9800));
pub const RNG_LCD_BGDD1: AddressRange = AddressRange(Address(0x9800), Address(0x9C00));
pub const RNG_LCD_BGDD2: AddressRange = AddressRange(Address(0x9C00), Address(0xA000));
pub const RNG_LCD_OAM: AddressRange = AddressRange(Address(0xFE00), Address(0xFEA0));
pub const RNG_SND_REGS: AddressRange = AddressRange(Address(0xFF10), Address(0xFF27));
pub const RNG_SND_WAV_RAM: AddressRange = AddressRange(Address(0xFF30), Address(0xFF3F));
const REG_DMA: Address = Address(0xFF46);

impl Mmu {
    pub fn new(cart: Cart) -> Mmu {
        Mmu {
            internal_ram: Ram::new(RNG_INT_RAM.len()),
            tiny_ram: Ram::new(RNG_INT_TINY_RAM.len()),
            interrupt_enable: 0,
            interrupt_table: Ram::new(RNG_INTR_TABLE.len()),
            cart,
            lcd: Box::new(Lcd::new()),
            audio: Audio::new(),
        }
    }

    fn dma(&mut self, mut src: Address) -> Result<(), ()> {
        // TODO: This should actually take 160us worth of cycles
        let mut dst = RNG_LCD_OAM.0;
        while dst < RNG_LCD_OAM.1 {
            let v = try!(self.read(src));
            try!(self.write(dst, v));
            dst += Address(1);
            src += Address(1);
        }

        Ok(())
    }
}

impl MemDevice for Mmu {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_INTR_TABLE) {
            self.interrupt_table.read(a - RNG_INTR_TABLE.0)
        } else if a.in_(RNG_INT_RAM) {
            self.internal_ram.read(a - RNG_INT_RAM.0)
        } else if a.in_(RNG_ROM_BANK0) || a.in_(RNG_ROM_BANK1) || a.in_(RNG_EXT_RAM) {
            self.cart.read(a)
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.read(a - RNG_INT_TINY_RAM.0)
        } else if a == OFF_INTR_ENABLE_REG {
            Ok(self.interrupt_enable)
        } else if a.in_(RNG_LCD_MM_REG) || a.in_(RNG_CHAR_DAT) || a.in_(RNG_LCD_BGDD1)
            || a.in_(RNG_LCD_BGDD2) || a.in_(RNG_LCD_OAM)
        {
            self.lcd.read(a)
        } else if a.in_(RNG_SND_WAV_RAM) || a.in_(RNG_SND_REGS) {
            self.audio.read(a)
        } else {
            println!("MMU: Unimplemented memory read at address {:?}", a);
            Err(())
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if a == REG_DMA {
            self.dma(Address((v as u16) << 8))
        } else if a.in_(RNG_INTR_TABLE) {
            self.interrupt_table.write(a - RNG_INTR_TABLE.0, v)
        } else if a.in_(RNG_INT_RAM) {
            self.internal_ram.write(a - RNG_INT_RAM.0, v)
        } else if a.in_(RNG_ROM_BANK0) || a.in_(RNG_ROM_BANK1) || a.in_(RNG_EXT_RAM) {
            self.cart.write(a, v)
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.write(a - RNG_INT_TINY_RAM.0, v)
        } else if a == OFF_INTR_ENABLE_REG {
            self.interrupt_enable = v;
            Ok(())
        } else if a.in_(RNG_LCD_MM_REG) || a.in_(RNG_CHAR_DAT) || a.in_(RNG_LCD_BGDD1)
            || a.in_(RNG_LCD_BGDD2) || a.in_(RNG_LCD_OAM)
        {
            self.lcd.write(a, v)
        } else if a.in_(RNG_SND_WAV_RAM) || a.in_(RNG_SND_REGS) {
            self.audio.write(a, v)
        } else {
            println!("MMU: Unimplemented memory write at address {:?}", a);
            Err(())
        }
    }
}
