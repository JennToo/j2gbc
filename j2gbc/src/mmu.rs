use std::collections::HashSet;

use log::{error, info};

use crate::alu::hi_lo;
use crate::audio::{Audio, AudioSink};
use crate::cart::Cart;
use crate::input::Input;
use crate::lcd::Lcd;
use crate::mem::*;
use crate::mmu_exceptions::MmuExceptions;
use crate::timer::Timer;

pub struct Mmu {
    internal_ram: Ram,
    tiny_ram: Ram,

    ram_bank_select: usize,

    pub double_speed_mode: bool,
    pub prepared_speed_switch: bool,

    pub cart: Cart,
    pub interrupt_enable: u8,
    pub interrupt_flag: u8,
    pub lcd: Box<Lcd>,
    pub audio: Audio,
    pub timer: Timer,
    pub input: Input,
    pub pedantic: bool,

    pub watchpoints: HashSet<Address>,

    exceptions: MmuExceptions,

    hdma1: u8,
    hdma2: u8,
    hdma3: u8,
    hdma4: u8,
    hdma5: u8,
}

impl Mmu {
    pub fn new(cart: Cart, audio_sink: Box<AudioSink>) -> Mmu {
        Mmu {
            internal_ram: Ram::new(RNG_INT_RAM_0.len() * 8),
            tiny_ram: Ram::new(RNG_INT_TINY_RAM.len()),
            double_speed_mode: false,
            prepared_speed_switch: false,
            interrupt_enable: 0,
            interrupt_flag: 0,
            exceptions: cart.get_mmu_exceptions(),
            cart,
            lcd: Box::new(Lcd::new()),
            audio: Audio::new(audio_sink),
            timer: Timer::new(),
            input: Input::new(),
            pedantic: true,
            ram_bank_select: 1,

            hdma1: 0,
            hdma2: 0,
            hdma3: 0,
            hdma4: 0,
            hdma5: 0,

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

    fn hdma(&mut self, src: AddressRange, mut dest: Address) -> Result<(), ()> {
        let mut src_cursor = src.0;
        while src_cursor < src.1 {
            let v = self.read(src_cursor)?;
            self.write(dest, v)?;
            dest += Address(1);
            src_cursor += Address(1);
        }
        Ok(())
    }

    fn _read(&self, a: Address) -> Result<u8, ()> {
        if self.watchpoints.contains(&a) {
            info!("Read watchpoint for {:?}", a);
            Err(())
        } else if a == REG_SVBK {
            Ok(self.ram_bank_select as u8)
        } else if a == REG_HDMA1 {
            Ok(self.hdma1)
        } else if a == REG_HDMA2 {
            Ok(self.hdma2)
        } else if a == REG_HDMA3 {
            Ok(self.hdma3)
        } else if a == REG_HDMA4 {
            Ok(self.hdma4)
        } else if a == REG_HDMA5 {
            Ok(self.hdma5)
        } else if a.in_(RNG_INT_RAM_0) {
            self.internal_ram.read(a - RNG_INT_RAM_0.0)
        } else if a.in_(RNG_INT_RAM_1) {
            self.internal_ram
                .read(ram_bank_adjust(a, self.ram_bank_select))
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
        } else if a == REG_RP {
            // IR not supported right now
            Ok(())
        } else if a == REG_DMA {
            self.dma(Address((u16::from(v)) << 8))
        } else if a == REG_HDMA1 {
            self.hdma1 = v;
            Ok(())
        } else if a == REG_HDMA2 {
            self.hdma2 = v;
            Ok(())
        } else if a == REG_HDMA3 {
            self.hdma3 = v;
            Ok(())
        } else if a == REG_HDMA4 {
            self.hdma4 = v;
            Ok(())
        } else if a == REG_HDMA5 {
            if v & 0b1000_0000 != 0 {
                panic!("HDMA HBlank not supported");
            }
            let src_start = Address(hi_lo(self.hdma1, self.hdma2) & 0b1111_1111_1111_0000);
            let src_end = src_start + Address(((v & 0b0111_1111) as u16 + 1) * 16);
            let dest =
                Address((hi_lo(self.hdma3, self.hdma4) & 0b0001_1111_1111_0000) | (0b1 << 15));
            self.hdma5 = 0xFF;
            self.hdma(AddressRange(src_start, src_end), dest)
        } else if a == REG_KEY1 {
            self.prepared_speed_switch = (0b1 & v) == 1;
            Ok(())
        } else if a == REG_SVBK {
            self.ram_bank_select = usize::from(v & 0b111);
            Ok(())
        } else if a.in_(RNG_INT_RAM_0) {
            self.internal_ram.write(a - RNG_INT_RAM_0.0, v)
        } else if a.in_(RNG_INT_RAM_1) {
            self.internal_ram
                .write(ram_bank_adjust(a, self.ram_bank_select), v)
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
        if self.pedantic && !self.exceptions.allow(a) {
            self._read(a)
        } else {
            self._read(a).or(Ok(0))
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if self.pedantic && !self.exceptions.allow(a) {
            self._write(a, v)
        } else {
            self._write(a, v).or(Ok(()))
        }
    }
}

fn ram_bank_adjust(a: Address, bank: usize) -> Address {
    let bank_offset =
        RNG_INT_RAM_1.len() * if bank > 0 { bank - 1 } else { 0 } + RNG_INT_RAM_0.len();
    (a - RNG_INT_RAM_1.0) + Address(bank_offset as u16)
}
