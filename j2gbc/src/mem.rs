use std::convert::Into;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

pub const RNG_INTR_TABLE: AddressRange = AddressRange(Address(0x0000), Address(0x0100));
pub const RNG_ROM_BANK0: AddressRange = AddressRange(Address(0x0100), Address(0x4000));
pub const RNG_ROM_BANK1: AddressRange = AddressRange(Address(0x4000), Address(0x8000));
pub const RNG_CHAR_DAT: AddressRange = AddressRange(Address(0x8000), Address(0x9800));
pub const RNG_LCD_BGDD1: AddressRange = AddressRange(Address(0x9800), Address(0x9C00));
pub const RNG_LCD_BGDD2: AddressRange = AddressRange(Address(0x9C00), Address(0xA000));
pub const RNG_EXT_RAM: AddressRange = AddressRange(Address(0xA000), Address(0xC000));
pub const RNG_INT_RAM_0: AddressRange = AddressRange(Address(0xC000), Address(0xD000));
pub const RNG_INT_RAM_1: AddressRange = AddressRange(Address(0xD000), Address(0xE000));
pub const RNG_LCD_OAM: AddressRange = AddressRange(Address(0xFE00), Address(0xFEA0));
pub const RNG_SND_REGS: AddressRange = AddressRange(Address(0xFF10), Address(0xFF27));
pub const RNG_SND_WAV_RAM: AddressRange = AddressRange(Address(0xFF30), Address(0xFF40));
pub const RNG_LCD_MM_REG: AddressRange = AddressRange(Address(0xFF40), Address(0xFF6C));
pub const RNG_INT_TINY_RAM: AddressRange = AddressRange(Address(0xFF80), Address(0xFFFF));

pub const REG_INTR_ENABLE: Address = Address(0xFFFF);
pub const REG_P1: Address = Address(0xFF00);
pub const REG_DMA: Address = Address(0xFF46);
pub const REG_KEY1: Address = Address(0xFF4D);
pub const REG_RP: Address = Address(0xFF56);
pub const REG_SVBK: Address = Address(0xFF70);
pub const REG_SB: Address = Address(0xFF01);
pub const REG_SC: Address = Address(0xFF02);
pub const REG_DIV: Address = Address(0xFF04);
pub const REG_TIMA: Address = Address(0xFF05);
pub const REG_TMA: Address = Address(0xFF06);
pub const REG_TAC: Address = Address(0xFF07);
pub const REG_INTR_FLAG: Address = Address(0xFF0F);

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash)]
pub struct Address(pub u16);

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash)]
pub struct ExtendedAddress(pub u32);

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Address({:#X})", self.0)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:04x}", self.0)
    }
}

impl Debug for ExtendedAddress {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ExtendedAddress({:#X})", self.0)
    }
}

impl Display for ExtendedAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:08x}", self.0)
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
        Address((i32::from(self.0) + i32::from(o)) as u16)
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

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone, Hash)]
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
        self.write(a, (v & 0xFF) as u8)?;
        self.write(a + Address(1), ((v >> 8) & 0xFF) as u8)?;
        Ok(())
    }

    fn read16(&self, a: Address) -> Result<u16, ()> {
        let hi = u16::from(self.read(a + Address(1))?);
        let lo = u16::from(self.read(a)?);
        Ok(hi << 8 | lo)
    }
}

#[derive(Clone, Debug)]
pub struct Ram {
    pub data: Vec<u8>,
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
