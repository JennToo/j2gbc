use emu::mem::{Address, AddressRange, MemDevice, RNG_ROM_BANK1, Ram, RNG_EXT_RAM};
use super::Mbc;

const RNG_BANK_SELECT: AddressRange = AddressRange(Address(0x2000), Address(0x4000));
const RNG_RAMCS: AddressRange = AddressRange(Address(0x0000), Address(0x2000));
const MASK_BANK_SELECT: u8 = 0b0001_1111;

pub struct Mbc1 {
    ram_protected: bool,
    rom: Vec<u8>,
    rom_bank: usize,
    ram: Ram,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>) -> Mbc1 {
        Mbc1 {
            ram_protected: true,
            rom,
            rom_bank: 1,
            ram: Ram::new(RNG_EXT_RAM.len()),
        }
    }
}

impl MemDevice for Mbc1 {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_ROM_BANK1) {
            let index = RNG_ROM_BANK1.len() * (self.rom_bank - 1) + a.0 as usize;
            Ok(self.rom[index])
        } else if a.in_(RNG_EXT_RAM) {
            self.ram.read(a - RNG_EXT_RAM.0)
        } else {
            unreachable!();
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if a.in_(RNG_BANK_SELECT) {
            self.rom_bank = (v & MASK_BANK_SELECT) as usize;
            Ok(())
        } else if a.in_(RNG_EXT_RAM) {
            if self.ram_protected {
                error!("Error: RAM is not writable right now");
                Err(())
            } else {
                self.ram.write(a - RNG_EXT_RAM.0, v)
            }
        } else if a.in_(RNG_RAMCS) {
            self.ram_protected = false;
            Ok(())
        } else {
            error!("Unimplemented MBC1 register");
            Err(())
        }
    }
}

impl Mbc for Mbc1 {}
