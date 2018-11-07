use std::collections::HashSet;

use log::{error, info};

use audio::{Audio, AudioSink};
use cart::Cart;
use input::Input;
use lcd::Lcd;
use mem::*;
use timer::Timer;

pub struct Mmu {
    internal_ram: Ram,
    tiny_ram: Ram,

    pub double_speed_mode: bool,
    pub prepared_speed_switch: bool,

    pub cart: Cart,
    pub interrupt_enable: u8,
    pub interrupt_flag: u8,
    pub lcd: Box<Lcd>,
    pub audio: Audio,
    pub timer: Timer,
    pub input: Input,

    pub watchpoints: HashSet<Address>,
}

impl Mmu {
    pub fn new(cart: Cart, audio_sink: Box<AudioSink>) -> Mmu {
        Mmu {
            internal_ram: Ram::new(RNG_INT_RAM.len()),
            tiny_ram: Ram::new(RNG_INT_TINY_RAM.len()),
            double_speed_mode: false,
            prepared_speed_switch: false,
            interrupt_enable: 0,
            interrupt_flag: 0,
            cart,
            lcd: Box::new(Lcd::new()),
            audio: Audio::new(audio_sink),
            timer: Timer::new(),
            input: Input::new(),

            watchpoints: HashSet::new(),
        }
    }

    fn dma(&mut self, mut src: Address) -> Result<(), ()> {
        // TODO: This should actually take 160us worth of cycles
        let mut dst = RNG_LCD_OAM.0;
        while dst < RNG_LCD_OAM.1 {
            let v = self.read(src)?;
            self.write(dst, v)?;
            dst += Address(1);
            src += Address(1);
        }

        Ok(())
    }

    fn _read(&self, a: Address) -> Result<u8, ()> {
        if self.watchpoints.contains(&a) {
            info!("Read watchpoint for {:?}", a);
            Err(())
        } else if a.in_(RNG_INT_RAM) {
            self.internal_ram.read(a - RNG_INT_RAM.0)
        } else if a == REG_KEY1 {
            let mode = if self.double_speed_mode {
                0b1000_0000
            } else {
                0
            };
            let prepared_mode = if self.prepared_speed_switch { 1 } else { 0 };
            Ok(mode | prepared_mode)
        } else if a.in_(RNG_ROM_BANK0)
            || a.in_(RNG_ROM_BANK1)
            || a.in_(RNG_EXT_RAM)
            || a.in_(RNG_INTR_TABLE)
        {
            self.cart.read(a)
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.read(a - RNG_INT_TINY_RAM.0)
        } else if a.in_(RNG_LCD_MM_REG)
            || a.in_(RNG_CHAR_DAT)
            || a.in_(RNG_LCD_BGDD1)
            || a.in_(RNG_LCD_BGDD2)
            || a.in_(RNG_LCD_OAM)
        {
            self.lcd.read(a)
        } else if a.in_(RNG_SND_WAV_RAM) || a.in_(RNG_SND_REGS) {
            self.audio.read(a)
        } else {
            match a {
                REG_INTR_ENABLE => Ok(self.interrupt_enable),
                REG_INTR_FLAG => Ok(self.interrupt_flag),
                REG_TIMA | REG_DIV | REG_TAC | REG_TMA => self.timer.read(a),
                REG_P1 => self.input.read(a),
                REG_SB | REG_SC => Ok(0),
                _ => {
                    error!("MMU: Unimplemented memory read at address {:?}", a);
                    Err(())
                }
            }
        }
    }

    fn _write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if self.watchpoints.contains(&a) {
            info!("Write watchpoint for {:?}", a);
            Err(())
        } else if a == REG_DMA {
            self.dma(Address((u16::from(v)) << 8))
        } else if a == REG_KEY1 {
            self.prepared_speed_switch = (0b1 & v) == 1;
            Ok(())
        } else if a.in_(RNG_INT_RAM) {
            self.internal_ram.write(a - RNG_INT_RAM.0, v)
        } else if a.in_(RNG_ROM_BANK0)
            || a.in_(RNG_ROM_BANK1)
            || a.in_(RNG_EXT_RAM)
            || a.in_(RNG_INTR_TABLE)
        {
            self.cart.write(a, v)
        } else if a.in_(RNG_INT_TINY_RAM) {
            self.tiny_ram.write(a - RNG_INT_TINY_RAM.0, v)
        } else if a.in_(RNG_LCD_MM_REG)
            || a.in_(RNG_CHAR_DAT)
            || a.in_(RNG_LCD_BGDD1)
            || a.in_(RNG_LCD_BGDD2)
            || a.in_(RNG_LCD_OAM)
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
                REG_TIMA | REG_DIV | REG_TAC | REG_TMA => self.timer.write(a, v),
                REG_P1 => self.input.write(a, v),
                REG_SB | REG_SC => Ok(()),
                _ => {
                    error!("MMU: Unimplemented memory write at address {:?}", a);
                    Err(())
                }
            }
        }
    }
}

impl MemDevice for Mmu {
    fn read(&self, a: Address) -> Result<u8, ()> {
        // self._read(a).or(Ok(0))
        self._read(a)
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        // self._write(a, v);
        // Ok(())
        self._write(a, v)
    }
}
