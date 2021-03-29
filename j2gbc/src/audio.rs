use log::error;

use super::mem::{Address, MemDevice, Ram, RNG_SND_WAV_RAM};
use crate::error::ExecutionError;

mod mixer;
mod noise;
mod square;
mod synth;
mod wave;

const REG_NR10: Address = Address(0xFF10);
const REG_NR11: Address = Address(0xFF11);
const REG_NR12: Address = Address(0xFF12);
const REG_NR13: Address = Address(0xFF13);
const REG_NR14: Address = Address(0xFF14);
const REG_NR21: Address = Address(0xFF16);
const REG_NR22: Address = Address(0xFF17);
const REG_NR23: Address = Address(0xFF18);
const REG_NR24: Address = Address(0xFF19);
const REG_NR30: Address = Address(0xFF1A);
const REG_NR31: Address = Address(0xFF1B);
const REG_NR32: Address = Address(0xFF1C);
const REG_NR33: Address = Address(0xFF1D);
const REG_NR34: Address = Address(0xFF1E);
const REG_NR41: Address = Address(0xFF20);
const REG_NR42: Address = Address(0xFF21);
const REG_NR43: Address = Address(0xFF22);
const REG_NR44: Address = Address(0xFF23);
const REG_NR50: Address = Address(0xFF24);
const REG_NR51: Address = Address(0xFF25);
const REG_NR52: Address = Address(0xFF26);

pub struct Audio {
    wav: Ram,
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
    nr50: u8,
    nr51: u8,
    nr52: u8,

    pub synth: synth::Synth,
}

pub trait AudioSink {
    fn emit_sample(&mut self, sample: (f32, f32));
    fn emit_raw_chans(&mut self, _chans: [f32; 4]) {}
    fn sample_rate(&self) -> u64;
}

pub struct NullSink;

impl AudioSink for NullSink {
    fn emit_sample(&mut self, _: (f32, f32)) {
        // Do nothing
    }

    fn sample_rate(&self) -> u64 {
        1
    }
}

impl Audio {
    pub fn new(sink: Box<dyn AudioSink + Send>) -> Audio {
        Audio {
            wav: Ram::new(RNG_SND_WAV_RAM.len()),
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,

            synth: synth::Synth::new(sink),
        }
    }
}

impl MemDevice for Audio {
    fn read(&self, a: Address) -> Result<u8, ExecutionError> {
        if a.in_(RNG_SND_WAV_RAM) {
            self.wav.read(a - RNG_SND_WAV_RAM.0)
        } else {
            match a {
                REG_NR10 => Ok(self.nr10),
                REG_NR11 => Ok(self.nr11),
                REG_NR12 => Ok(self.nr12),
                REG_NR14 => Ok(self.nr14),
                REG_NR21 => Ok(self.nr21),
                REG_NR22 => Ok(self.nr22),
                REG_NR23 => Ok(self.nr23),
                REG_NR24 => Ok(self.nr24),
                REG_NR30 => Ok(self.nr30),
                REG_NR31 => Ok(self.nr31),
                REG_NR32 => Ok(self.nr32),
                REG_NR33 => Ok(self.nr33),
                REG_NR34 => Ok(self.nr34),
                REG_NR41 => Ok(self.nr41),
                REG_NR42 => Ok(self.nr42),
                REG_NR43 => Ok(self.nr43),
                REG_NR44 => Ok(self.nr44),
                REG_NR50 => Ok(self.nr50),
                REG_NR51 => Ok(self.nr51),
                REG_NR52 => {
                    let mut v = 0b1000_0000 & self.nr52;
                    if self.synth.chan1.is_active() {
                        v |= 0b0000_0001;
                    }
                    if self.synth.chan2.is_active() {
                        v |= 0b0000_0010;
                    }
                    if self.synth.chan3.is_active() {
                        v |= 0b0000_0100;
                    }
                    if self.synth.chan4.is_active() {
                        v |= 0b0000_1000;
                    }
                    Ok(v)
                }
                _ => {
                    error!("Unimplemented sound register {:?}", a);
                    Err(ExecutionError::BusError)
                }
            }
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ExecutionError> {
        if a.in_(RNG_SND_WAV_RAM) {
            let offset = a - RNG_SND_WAV_RAM.0;
            self.wav.write(offset, v).unwrap();
            self.synth
                .chan3
                .write_sample(bits_to_sample(v & 0b1111), offset.0 as usize * 2 + 1);
            self.synth
                .chan3
                .write_sample(bits_to_sample(v >> 4), offset.0 as usize * 2);
            Ok(())
        } else {
            match a {
                REG_NR10 => {
                    self.nr10 = v;
                    self.synth.chan1.set_freqeuncy_sweepers(
                        v >> 4 & 0b111,
                        v & 0b111,
                        v & 0b1000 == 0,
                    );
                    Ok(())
                }
                REG_NR11 => {
                    self.nr11 = v;
                    self.synth.chan1.set_duty_cycle(v >> 6);
                    self.synth.chan1.update_length(v & 0b0011_1111);
                    Ok(())
                }
                REG_NR12 => {
                    self.nr12 = v;
                    self.synth.chan1.set_vol_env_period(v & 0b111);
                    self.synth.chan1.increment_vol_env(v & 0b1000 != 0);
                    self.synth.chan1.set_volume(v >> 4);
                    Ok(())
                }
                REG_NR13 => {
                    self.nr13 = v;
                    self.synth
                        .chan1
                        .set_frequency_from_bits(self.nr14, self.nr13);
                    Ok(())
                }
                REG_NR14 => {
                    self.nr14 = v;
                    self.synth
                        .chan1
                        .set_frequency_from_bits(self.nr14, self.nr13);
                    self.synth.chan1.use_length_counter(v & 0b0100_0000 != 0);
                    if v & 0b1000_0000 != 0 {
                        self.synth.chan1.reset();
                    }
                    Ok(())
                }
                REG_NR21 => {
                    self.nr21 = v;
                    self.synth.chan2.set_duty_cycle(v >> 6);
                    self.synth.chan2.update_length(v & 0b0011_1111);
                    Ok(())
                }
                REG_NR22 => {
                    self.nr22 = v;
                    self.synth.chan2.set_vol_env_period(v & 0b111);
                    self.synth.chan2.increment_vol_env(v & 0b1000 != 0);
                    self.synth.chan2.set_volume(v >> 4);
                    Ok(())
                }
                REG_NR23 => {
                    self.nr23 = v;
                    self.synth
                        .chan2
                        .set_frequency_from_bits(self.nr24, self.nr23);
                    Ok(())
                }
                REG_NR24 => {
                    self.nr24 = v;
                    self.synth
                        .chan2
                        .set_frequency_from_bits(self.nr24, self.nr23);
                    self.synth.chan2.use_length_counter(v & 0b0100_0000 != 0);
                    if v & 0b1000_0000 != 0 {
                        self.synth.chan2.reset();
                    }
                    Ok(())
                }
                REG_NR30 => {
                    self.nr30 = v;
                    self.synth.chan3.enabled = v & 0b1000_0000 != 0;
                    Ok(())
                }
                REG_NR31 => {
                    self.nr31 = v;
                    self.synth.chan3.set_len(v);
                    Ok(())
                }
                REG_NR32 => {
                    self.nr32 = v;
                    self.synth.chan3.vol_multiplier = match (v >> 5) & 0b11 {
                        0 => 0.,
                        1 => 1.,
                        2 => 0.5,
                        3 => 0.25,
                        _ => unreachable!(),
                    };
                    Ok(())
                }
                REG_NR33 => {
                    self.nr33 = v;
                    self.synth
                        .chan3
                        .set_frequency_from_bits(self.nr34, self.nr33);
                    Ok(())
                }
                REG_NR34 => {
                    self.nr34 = v;
                    self.synth
                        .chan3
                        .set_frequency_from_bits(self.nr34, self.nr33);
                    self.synth.chan3.use_len = v & 0b0100_0000 != 0;
                    if v & 0b1000_0000 != 0 {
                        self.synth.chan3.reset();
                    }
                    Ok(())
                }
                REG_NR41 => {
                    self.nr41 = v;
                    self.synth.chan4.set_len(v & 0b0011_1111);
                    Ok(())
                }
                REG_NR42 => {
                    self.nr42 = v;
                    self.synth.chan4.set_vol_env_period(v & 0b111);
                    self.synth.chan4.increment_vol_env(v & 0b1000 != 0);
                    self.synth.chan4.set_volume(v >> 4);
                    Ok(())
                }
                REG_NR43 => {
                    self.nr43 = v;
                    self.synth.chan4.set_frequency_from_bits(v);
                    Ok(())
                }
                REG_NR44 => {
                    self.nr44 = v;
                    self.synth.chan4.use_len = v & 0b0100_0000 != 0;
                    if v & 0b1000_0000 != 0 {
                        self.synth.chan4.reset();
                    }
                    Ok(())
                }
                REG_NR50 => {
                    self.nr50 = v;
                    self.synth.mixer.set_master_volumes(
                        f32::from((v >> 4) & 0b111) / 7.,
                        f32::from(v & 0b111) / 7.,
                    );
                    Ok(())
                }
                REG_NR51 => {
                    self.nr51 = v;
                    self.synth.mixer.set_enabled_channels(
                        [
                            v & 0b0000_0001 != 0,
                            v & 0b0000_0010 != 0,
                            v & 0b0000_0100 != 0,
                            v & 0b0000_1000 != 0,
                        ],
                        [
                            v & 0b0001_0000 != 0,
                            v & 0b0010_0000 != 0,
                            v & 0b0100_0000 != 0,
                            v & 0b1000_0000 != 0,
                        ],
                    );
                    Ok(())
                }
                REG_NR52 => {
                    self.nr52 = v;
                    Ok(())
                }
                _ => {
                    error!("Unimplemented sound register {:?}", a);
                    Err(ExecutionError::BusError)
                }
            }
        }
    }
}

fn bits_to_sample(b: u8) -> f32 {
    (f32::from(b) - 8.) / 8.
}

#[test]
fn test_bits_to_sample() {
    assert_eq!(bits_to_sample(0), -1.);
    assert_eq!(bits_to_sample(8), 0.);
    assert_eq!(bits_to_sample(16), 1.);
}
