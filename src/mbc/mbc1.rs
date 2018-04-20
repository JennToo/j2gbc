use mem::{Address, AddressRange, MemDevice, RNG_ROM_BANK1};
use mbc::Mbc;

const RNG_BANK_SELECT: AddressRange = AddressRange(Address(0x2000), Address(0x4000));
const MASK_BANK_SELECT: u8 = 0b00011111;

pub struct Mbc1 {
    rom: Vec<u8>,
    rom_bank: usize,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>) -> Mbc1 {
        Mbc1 { rom, rom_bank: 1 }
    }
}

impl MemDevice for Mbc1 {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_ROM_BANK1) {
            let index = RNG_ROM_BANK1.len() * self.rom_bank;
            Ok(self.rom[index])
        } else {
            unreachable!();
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if a.in_(RNG_BANK_SELECT) {
            self.rom_bank = (v & MASK_BANK_SELECT) as usize;
            Ok(())
        } else {
            println!("Unimplemented MBC1 register");
            Err(())
        }
    }
}

impl Mbc for Mbc1 {}
