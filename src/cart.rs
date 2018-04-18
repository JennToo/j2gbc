use std::io::Read;
use std::io;
use mem::{Address, MemDevice, RNG_ROM_BANK0};

pub struct Cart {
    pub data: Vec<u8>,
}

const OFF_CART_NAME_START: usize = 0x134;
const OFF_CART_NAME_END: usize = 0x142;
const OFF_CART_TYPE: usize = 0x147;
const OFF_CART_SIZE: usize = 0x148;
const OFF_RAM_SIZE: usize = 0x149;

impl Cart {
    pub fn load<R: Read>(mut r: R) -> io::Result<Cart> {
        let mut data = Vec::new();
        try!(r.read_to_end(&mut data));
        Ok(Cart { data })
    }

    pub fn name(&self) -> String {
        let s = &self.data.as_slice()[OFF_CART_NAME_START..OFF_CART_NAME_END];
        String::from_utf8_lossy(s).into_owned()
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
            3 => 32768,
            4 => 131072,
            _ => unimplemented!(),
        }
    }
}

impl MemDevice for Cart {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_ROM_BANK0) {
            Ok(self.data[a.0 as usize])
        } else {
            println!("Unimplemented region for cart {:#X}", a.0);
            Err(())
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if a.in_(RNG_ROM_BANK0) {
            self.data[a.0 as usize] = v;
            Ok(())
        } else {
            println!("Unimplemented region for cart {:#X}", a.0);
            Err(())
        }
    }
}
