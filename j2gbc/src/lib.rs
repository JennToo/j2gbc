#![allow(unknown_lints)]
#![allow(clippy::upper_case_acronyms)]

mod alu;
mod audio;
mod cart;
mod cpu;
pub mod debug;
mod error;
mod input;
mod inst;
mod lcd;
mod mbc;
mod mem;
mod mmu;
mod mmu_exceptions;
mod system;
mod timer;

pub use crate::{
    audio::{AudioSink, NullSink},
    input::Button,
    lcd::fb::{Framebuffer, SCREEN_SIZE},
    system::System,
};
