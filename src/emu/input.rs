use std::collections::HashSet;
use std::ops::BitOr;

use super::mem::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Button {
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}

const P10: u8 = 0b0000_0001;
const P11: u8 = 0b0000_0010;
const P12: u8 = 0b0000_0100;
const P13: u8 = 0b0000_1000;

const P14: u8 = 0b0001_0000;
const P15: u8 = 0b0010_0000;

const INPUT_MASK: u8 = P10 | P11 | P12 | P13;
const OUTPUT_MASK: u8 = P14 | P15;

impl Button {
    fn selected_by_output(&self, output: u8) -> bool {
        match self {
            Button::Left | Button::Right | Button::Down | Button::Up => (output & P14) == 0,
            Button::A | Button::B | Button::Start | Button::Select => (output & P15) == 0,
        }
    }

    fn output(&self) -> u8 {
        match self {
            Button::Right | Button::A => P10,
            Button::Left | Button::B => P11,
            Button::Up | Button::Select => P12,
            Button::Down | Button::Start => P13,
        }
    }
}

pub struct Input {
    active: HashSet<Button>,
    p1: u8,
}

impl Input {
    pub fn new() -> Input {
        Input {
            active: HashSet::new(),
            p1: OUTPUT_MASK | INPUT_MASK,
        }
    }

    fn active_input_bits(&self, output_bits: u8) -> u8 {
        (!self
            .active
            .iter()
            .filter(|x| x.selected_by_output(output_bits))
            .map(Button::output)
            .fold(0, u8::bitor)) & INPUT_MASK
    }

    fn recalculate(&mut self) {
        let output_bits = self.p1 & OUTPUT_MASK;
        let input_bits = self.active_input_bits(output_bits);
        self.p1 = output_bits | input_bits;
    }

    pub fn activate_button(&mut self, button: Button) {
        self.active.insert(button);
        self.recalculate();
    }

    pub fn deactivate_button(&mut self, button: Button) {
        self.active.remove(&button);
        self.recalculate();
    }
}

impl MemDevice for Input {
    fn read(&self, a: Address) -> Result<u8, ()> {
        assert_eq!(a, REG_P1);

        Ok(self.p1)
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        assert_eq!(a, REG_P1);

        self.p1 = v;
        self.recalculate();

        Ok(())
    }
}
