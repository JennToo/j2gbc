use j2ds::{Timer, TimerEvent};

use super::{LINE_CYCLE_TIME, LYC_MATCH_FLAG, LYC_MATCH_INT_FLAG, TOTAL_SCANLINES};
use crate::cpu::Interrupt;

pub struct ScanlineSweeper {
    ly: u8,
    lyc: u8,
    interrupt_enabled: bool,
    timer: Timer,
}

impl ScanlineSweeper {
    pub fn new() -> ScanlineSweeper {
        let mut timer = Timer::new(LINE_CYCLE_TIME, 0, 0);
        timer.update(0);
        ScanlineSweeper {
            ly: 0,
            lyc: 0,
            interrupt_enabled: false,
            timer,
        }
    }

    pub fn pump_cycle(&mut self, cycle: u64) -> Option<Interrupt> {
        if self.timer.update(cycle) == Some(TimerEvent::RisingEdge) {
            assert_eq!(self.timer.update(cycle), None); // We should never end up too far behind
            self.ly = (self.ly + 1) % TOTAL_SCANLINES as u8;

            if self.ly == self.lyc && self.interrupt_enabled {
                Some(Interrupt::LCDC)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn ly(&self) -> u8 {
        self.ly
    }

    pub fn lyc(&self) -> u8 {
        self.lyc
    }

    pub fn set_lyc(&mut self, v: u8) {
        self.lyc = v;
    }

    pub fn stat_flags(&self) -> u8 {
        if self.ly == self.lyc {
            LYC_MATCH_FLAG
        } else {
            0
        }
    }

    pub fn update_stat(&mut self, flags: u8) {
        self.interrupt_enabled = (flags & LYC_MATCH_INT_FLAG) != 0;
    }

    pub fn timer(&self) -> Timer {
        self.timer
    }

    pub fn on_visible_scanline(&self) -> bool {
        (self.ly as usize) < super::fb::SCREEN_SIZE.1
    }
}

#[test]
fn test_sweep_and_wrap() {
    let mut sweeper = ScanlineSweeper::new();

    assert_eq!(sweeper.ly(), 0);
    sweeper.pump_cycle(LINE_CYCLE_TIME - 1);
    assert_eq!(sweeper.ly(), 0);

    for scanline in 1..TOTAL_SCANLINES {
        sweeper.pump_cycle(LINE_CYCLE_TIME * scanline);
        assert_eq!(sweeper.ly(), scanline as u8);
    }

    sweeper.pump_cycle(LINE_CYCLE_TIME * TOTAL_SCANLINES);
    assert_eq!(sweeper.ly(), 0);
}

#[test]
fn test_lyc() {
    let mut sweeper = ScanlineSweeper::new();
    sweeper.set_lyc(42);
    for scanline in 0..42 {
        assert_eq!(sweeper.stat_flags(), 0);
        assert_eq!(sweeper.pump_cycle(LINE_CYCLE_TIME * scanline), None);
    }

    sweeper.update_stat(LYC_MATCH_INT_FLAG);
    assert_eq!(
        sweeper.pump_cycle(LINE_CYCLE_TIME * 42),
        Some(Interrupt::LCDC)
    );
    assert_eq!(sweeper.stat_flags(), LYC_MATCH_FLAG);
}
