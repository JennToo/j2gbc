use std::cmp::min;
use std::num::Wrapping;

use super::cpu::{Interrupt, CLOCK_RATE};
use super::mem::*;

const DIV_INCREMENT_CYCLE_COUNT: u64 = CLOCK_RATE / 16_779;
const TIMA_INCREMENT_CYCLE_COUNT: [u64; 4] = [
    CLOCK_RATE / 4_096,
    CLOCK_RATE / 262_144,
    CLOCK_RATE / 65_536,
    CLOCK_RATE / 16_384,
];

pub struct Timer {
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,

    double_speed: bool,

    next_div_cycle: u64,
    next_tima_cycle: u64,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,

            double_speed: false,

            next_div_cycle: 0,
            next_tima_cycle: 0,
        }
    }

    pub fn toggle_double_speed(&mut self) {
        self.double_speed = !self.double_speed;
        println!("Thanks Obama {}", self.double_speed);
    }

    fn tima_enabled(&self) -> bool {
        self.tac & 0b100 != 0
    }

    fn tima_duration(&self) -> u64 {
        TIMA_INCREMENT_CYCLE_COUNT[(self.tac & 0b11) as usize]
    }

    pub fn get_next_event_cycle(&self) -> u64 {
        if self.tima_enabled() {
            min(self.next_div_cycle, self.next_tima_cycle)
        } else {
            self.next_div_cycle
        }
    }

    pub fn pump_cycle(&mut self, cycle: u64) -> Option<Interrupt> {
        if self.next_div_cycle <= cycle {
            self.next_div_cycle =
                cycle + maybe_half_cycle(DIV_INCREMENT_CYCLE_COUNT, self.double_speed);
            self.div = (Wrapping(self.div) + Wrapping(1)).0;
        }

        if self.tima_enabled() && self.next_tima_cycle <= cycle {
            self.next_tima_cycle =
                cycle + maybe_half_cycle(self.tima_duration(), self.double_speed);

            if self.tima == 0xFF {
                self.tima = self.tma;
                Some(Interrupt::Timer)
            } else {
                self.tima += 1;
                None
            }
        } else {
            None
        }
    }
}

fn maybe_half_cycle(cycle_count: u64, double_speed: bool) -> u64 {
    if double_speed {
        cycle_count >> 2
    } else {
        cycle_count
    }
}

impl MemDevice for Timer {
    fn read(&self, a: Address) -> Result<u8, ()> {
        match a {
            REG_DIV => Ok(self.div),
            REG_TIMA => Ok(self.tima),
            REG_TMA => Ok(self.tma),
            REG_TAC => Ok(self.tac),
            _ => unreachable!(),
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        match a {
            REG_DIV => {
                self.div = 0;
            }
            REG_TIMA => {
                self.tima = v;
            }
            REG_TMA => {
                self.tma = v;
            }
            REG_TAC => {
                self.tac = v;
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}
