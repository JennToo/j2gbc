use log::error;

use super::Mbc;
use mem::{Address, AddressRange, ExtendedAddress, MemDevice, Ram, RNG_EXT_RAM, RNG_ROM_BANK1};

const RNG_RAMG: AddressRange = AddressRange(Address(0x0000), Address(0x2000));
const RNG_LOWER_BANK_SELECT: AddressRange = AddressRange(Address(0x2000), Address(0x3000));
const RNG_UPPER_BANK_SELECT: AddressRange = AddressRange(Address(0x3000), Address(0x4000));
const RNG_RAMB: AddressRange = AddressRange(Address(0x4000), Address(0x6000));

pub struct Mbc5 {
    ram_protected: bool,
    rom: Vec<u8>,
    rom_bank_select: usize,
    ram_bank_select: usize,
    ram: Ram,
}

impl Mbc5 {
    pub fn new(rom: Vec<u8>) -> Mbc5 {
        Mbc5 {
            ram_protected: true,
            rom,
            rom_bank_select: 1,
            ram_bank_select: 0,
            ram: Ram::new(RNG_EXT_RAM.len() * 16),
        }
    }
}

impl MemDevice for Mbc5 {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_ROM_BANK1) {
            let index = self.map_address_into_rom(a).0 as usize;
            Ok(self.rom[index])
        } else if a.in_(RNG_EXT_RAM) {
            self.ram.read(ram_bank_adjust(a, self.ram_bank_select))
        } else {
            unreachable!();
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if a.in_(RNG_EXT_RAM) {
            self.ram.write(ram_bank_adjust(a, self.ram_bank_select), v)
        } else if a.in_(RNG_RAMG) {
            self.ram_protected = v != 0x0A;
            Ok(())
        } else if a.in_(RNG_UPPER_BANK_SELECT) {
            self.rom_bank_select =
                ((v as usize) << 8 & 0b11) | (self.rom_bank_select & !(0b11_0000_0000));
            if self.rom_bank_select == 0 {
                self.rom_bank_select = 1;
            }
            Ok(())
        } else if a.in_(RNG_LOWER_BANK_SELECT) {
            self.rom_bank_select = (v as usize) | (self.rom_bank_select & !(0b1111_1111));
            if self.rom_bank_select == 0 {
                self.rom_bank_select = 1;
            }
            Ok(())
        } else if a.in_(RNG_RAMB) {
            self.ram_bank_select = (v & 0b1111) as usize;
            Ok(())
        } else {
            error!("Unimplemented MBC5 register {}", a);
            Err(())
        }
    }
}

impl Mbc for Mbc5 {
    fn map_address_into_rom(&self, a: Address) -> ExtendedAddress {
        ExtendedAddress((RNG_ROM_BANK1.len() * (self.rom_bank_select - 1)) as u32 + u32::from(a.0))
    }

    fn get_sram(&self) -> &[u8] {
        self.ram.data.as_slice()
    }

    fn set_sram(&mut self, buf: &[u8]) {
        self.ram.data[..buf.len()].clone_from_slice(buf);
    }
}

fn ram_bank_adjust(a: Address, bank: usize) -> Address {
    Address(((a - RNG_EXT_RAM.0).0 as usize + RNG_EXT_RAM.len() * bank) as u16)
}
