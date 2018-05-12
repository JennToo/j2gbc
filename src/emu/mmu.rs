use std::collections::HashSet;

use super::audio::Audio;
use super::cart::Cart;
use super::lcd::Lcd;
use super::mem::*;

pub struct Mmu {
    internal_ram: Ram,
    tiny_ram: Ram,
    pub cart: Cart,
    pub interrupt_enable: u8,
    interrupt_flag: u8,
    pub lcd: Box<Lcd>,
    audio: Audio,

    tma: u8,
    tac: u8,

    pub watchpoints: HashSet<Address>,
}

impl Mmu {
    pub fn new(cart: Cart) -> Mmu {
        Mmu {
            internal_ram: Ram::new(RNG_INT_RAM.len()),
            tiny_ram: Ram::new(RNG_INT_TINY_RAM.len()),
            interrupt_enable: 0,
            interrupt_flag: 0,
            tma: 0,
            tac: 0,
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
            info!("Read watchpoint for {:?}", a);
            Err(())
        } else if a.in_(RNG_INT_RAM) {
            self.internal_ram.read(a - RNG_INT_RAM.0)
        } else if a.in_(RNG_ROM_BANK0) || a.in_(RNG_ROM_BANK1) || a.in_(RNG_EXT_RAM)
            || a.in_(RNG_INTR_TABLE)
        {
            self.cart.read(a)
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.read(a - RNG_INT_TINY_RAM.0)
        } else if a.in_(RNG_LCD_MM_REG) || a.in_(RNG_CHAR_DAT) || a.in_(RNG_LCD_BGDD1)
            || a.in_(RNG_LCD_BGDD2) || a.in_(RNG_LCD_OAM)
        {
            self.lcd.read(a)
        } else if a.in_(RNG_SND_WAV_RAM) || a.in_(RNG_SND_REGS) {
            self.audio.read(a)
        } else {
            match a {
                REG_INTR_ENABLE => Ok(self.interrupt_enable),
                REG_INTR_FLAG => Ok(self.interrupt_flag),
                REG_TMA => Ok(self.tma),
                REG_TAC => Ok(self.tac),
                REG_P1 | REG_SB | REG_SC => Ok(0),
                _ => {
                    error!("MMU: Unimplemented memory read at address {:?}", a);
                    Err(())
                }
            }
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if self.watchpoints.contains(&a) {
            info!("Write watchpoint for {:?}", a);
            Err(())
        } else if a == REG_DMA {
            self.dma(Address((u16::from(v)) << 8))
        } else if a.in_(RNG_INT_RAM) {
            self.internal_ram.write(a - RNG_INT_RAM.0, v)
        } else if a.in_(RNG_ROM_BANK0) || a.in_(RNG_ROM_BANK1) || a.in_(RNG_EXT_RAM)
            || a.in_(RNG_INTR_TABLE)
        {
            self.cart.write(a, v)
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.write(a - RNG_INT_TINY_RAM.0, v)
        } else if a.in_(RNG_LCD_MM_REG) || a.in_(RNG_CHAR_DAT) || a.in_(RNG_LCD_BGDD1)
            || a.in_(RNG_LCD_BGDD2) || a.in_(RNG_LCD_OAM)
        {
            self.lcd.write(a, v)
        } else if a.in_(RNG_SND_WAV_RAM) || a.in_(RNG_SND_REGS) {
            self.audio.write(a, v)
        } else {
            match a {
                REG_INTR_ENABLE => {
                    self.interrupt_enable = v;
                    Ok(())
                }
                REG_INTR_FLAG => {
                    self.interrupt_flag = v;
                    Ok(())
                }
                REG_TMA => {
                    self.tma = v;
                    Ok(())
                }
                REG_TAC => {
                    self.tac = v;
                    Ok(())
                }
                REG_P1 | REG_SB | REG_SC => Ok(()),
                _ => {
                    error!("MMU: Unimplemented memory write at address {:?}", a);
                    Err(())
                }
            }
        }
    }
}
