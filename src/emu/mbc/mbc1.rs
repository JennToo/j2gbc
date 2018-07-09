use super::Mbc;
use emu::mem::{
    Address, AddressRange, ExtendedAddress, MemDevice, RNG_ROM_BANK1, Ram, RNG_EXT_RAM,
};

const RNG_LOWER_BANK_SELECT: AddressRange = AddressRange(Address(0x2000), Address(0x4000));
const RNG_RAMCS: AddressRange = AddressRange(Address(0x0000), Address(0x2000));
const RNG_UPPER_BANK_SELECT: AddressRange = AddressRange(Address(0x4000), Address(0x6000));
const RNG_CTRL_UPPER_BANK_SELECT: AddressRange = AddressRange(Address(0x6000), Address(0x8000));
const MAKS_UPPER_BANK_SELCET: u8 = 0b0000_0011;
const MASK_LOWER_BANK_SELECT: u8 = 0b0001_1111;

pub struct Mbc1 {
    ram_protected: bool,
    rom: Vec<u8>,
    lower_bank_select: usize,
    upper_bank_controls_rom: bool,
    upper_bank_select: usize,
    ram: Ram,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>) -> Mbc1 {
        Mbc1 {
            ram_protected: true,
            rom,
            upper_bank_controls_rom: true,
            upper_bank_select: 0,
            lower_bank_select: 1,
            ram: Ram::new(RNG_EXT_RAM.len() * 4),
        }
    }

    fn map_address_into_ram(&self, a: Address) -> Address {
        let bank = if !self.upper_bank_controls_rom {
            self.upper_bank_select
        } else {
            0
        };
        Address(((a - RNG_EXT_RAM.0).0 as usize + RNG_EXT_RAM.len() * bank) as u16)
    }
}

impl MemDevice for Mbc1 {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_ROM_BANK1) {
            let index = self.map_address_into_rom(a).0 as usize;
            Ok(self.rom[index])
        } else if a.in_(RNG_EXT_RAM) {
            self.ram.read(a - RNG_EXT_RAM.0)
        } else {
            unreachable!();
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if a.in_(RNG_LOWER_BANK_SELECT) {
            self.lower_bank_select = (v & MASK_LOWER_BANK_SELECT) as usize;
            if self.lower_bank_select == 0 {
                self.lower_bank_select = 1;
            }
            Ok(())
        } else if a.in_(RNG_EXT_RAM) {
            if self.ram_protected {
                error!("Error: RAM is not writable right now");
                Err(())
            } else {
                let mapped = self.map_address_into_ram(a);
                self.ram.write(mapped, v)
            }
        } else if a.in_(RNG_RAMCS) {
            self.ram_protected = !(v == 0x0A);
            Ok(())
        } else if a.in_(RNG_UPPER_BANK_SELECT) {
            self.upper_bank_select = (v & MAKS_UPPER_BANK_SELCET) as usize;
            Ok(())
        } else if a.in_(RNG_CTRL_UPPER_BANK_SELECT) {
            self.upper_bank_controls_rom = v == 0;
            Ok(())
        } else {
            error!("Unimplemented MBC1 register");
            Err(())
        }
    }
}

impl Mbc for Mbc1 {
    fn map_address_into_rom(&self, a: Address) -> ExtendedAddress {
        if self.upper_bank_controls_rom && self.upper_bank_select != 0 {
            unimplemented!();
        }

        ExtendedAddress((RNG_ROM_BANK1.len() * (self.lower_bank_select - 1)) as u32 + a.0 as u32)
    }

    fn get_sram(&self) -> &[u8] {
        self.ram.data.as_slice()
    }

    fn set_sram(&mut self, buf: &[u8]) {
        for i in 0..buf.len() {
            self.ram.data[i] = buf[i];
        }
    }
}
