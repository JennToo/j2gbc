use std::collections::HashSet;

use super::mem::*;
use super::cart::Cart;
use super::lcd::Lcd;
use super::audio::Audio;

pub struct Mmu {
    internal_ram: Ram,
    tiny_ram: Ram,
    cart: Cart,
    pub interrupt_enable: u8,
    pub interrupt_table: Ram,
    pub lcd: Box<Lcd>,
    audio: Audio,

    pub watchpoints: HashSet<Address>,
}

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

            watchpoints: HashSet::new(),
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
        if self.watchpoints.contains(&a) {
            println!("Read watchpoint for {:?}", a);
            Err(())
        } else if a.in_(RNG_INTR_TABLE) {
            self.interrupt_table.read(a - RNG_INTR_TABLE.0)
        } else if a.in_(RNG_INT_RAM) {
            self.internal_ram.read(a - RNG_INT_RAM.0)
        } else if a.in_(RNG_ROM_BANK0) || a.in_(RNG_ROM_BANK1) || a.in_(RNG_EXT_RAM) {
            self.cart.read(a)
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.read(a - RNG_INT_TINY_RAM.0)
        } else if a == REG_INTR_ENABLE {
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
        if self.watchpoints.contains(&a) {
            println!("Write watchpoint for {:?}", a);
            Err(())
        } else if a == REG_DMA {
            self.dma(Address((v as u16) << 8))
        } else if a.in_(RNG_INTR_TABLE) {
            self.interrupt_table.write(a - RNG_INTR_TABLE.0, v)
        } else if a.in_(RNG_INT_RAM) {
            self.internal_ram.write(a - RNG_INT_RAM.0, v)
        } else if a.in_(RNG_ROM_BANK0) || a.in_(RNG_ROM_BANK1) || a.in_(RNG_EXT_RAM) {
            self.cart.write(a, v)
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.write(a - RNG_INT_TINY_RAM.0, v)
        } else if a == REG_INTR_ENABLE {
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
