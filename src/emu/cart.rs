use std::io;
use std::io::Read;

use super::mbc::mbc1::Mbc1;
use super::mbc::Mbc;
use super::mem::{Address, ExtendedAddress, MemDevice, RNG_ROM_BANK0, RNG_ROM_BANK1, RNG_INTR_TABLE};

pub struct Cart {
    pub data: Vec<u8>,
    mbc: Box<Mbc>,
}

const OFF_CART_NAME_START: usize = 0x134;
const OFF_CART_NAME_END: usize = 0x142;
const OFF_CART_TYPE: usize = 0x147;
const OFF_CART_SIZE: usize = 0x148;
const OFF_RAM_SIZE: usize = 0x149;

impl Cart {
    pub fn load<R: Read>(mut r: R) -> io::Result<Cart> {
        let mut data = Vec::new();
        r.read_to_end(&mut data)?;
        Ok(Cart {
            data: data.clone(),
            mbc: Box::new(Mbc1::new(data)),
        })
    }

    pub fn name(&self) -> String {
        let b = &self.data.as_slice()[OFF_CART_NAME_START..OFF_CART_NAME_END];
        let s = b.iter()
            .take_while(|n| **n != 0)
            .map(|n| *n)
            .collect::<Vec<u8>>();
        String::from_utf8_lossy(&s[..]).into_owned()
    }

    pub fn type_(&self) -> u8 {
        self.data[OFF_CART_TYPE]
    }

    pub fn rom_size(&self) -> usize {
        32768 << self.data[OFF_CART_SIZE]
    }

    pub fn ram_size(&self) -> usize {
        match self.data[OFF_RAM_SIZE] {
            0 => 0,
            1 => 2048,
            2 => 8192,
            3 => 32_768,
            4 => 131_072,
            _ => unimplemented!(),
        }
    }

    pub fn map_address_into_rom(&self, a: Address) -> ExtendedAddress {
        if a.in_(RNG_ROM_BANK1) {
            self.mbc.map_address_into_rom(a)
        } else {
            ExtendedAddress(u32::from(a.0))
        }
    }
}

impl MemDevice for Cart {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_ROM_BANK0) || a.in_(RNG_INTR_TABLE) {
            Ok(self.data[a.0 as usize])
        } else {
            self.mbc.read(a)
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        self.mbc.write(a, v)
    }
}
