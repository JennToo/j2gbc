use std::cmp::min;

use super::cpu::CLOCK_RATE;
use super::mem::{Address, MemDevice, Ram, RNG_SND_WAV_RAM};

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

    sink: Box<AudioSink>,
    sink_rate: u64,

    next_sample_clock: u64,
    next_len_clock: u64,
    next_env_clock: u64,
    next_freq_clock: u64,

    audio_cycle: u64,

    chan1: SquareChannel,
    chan2: SquareChannel,
}

pub trait AudioSink {
    fn emit_sample(&mut self, sample: (f32, f32));
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
    pub fn new(sink: Box<AudioSink>) -> Audio {
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

            sink_rate: sink.sample_rate(),
            sink,

            next_sample_clock: 0,
            next_len_clock: 0,
            next_env_clock: 0,
            next_freq_clock: 0,

            audio_cycle: 0,

            chan1: SquareChannel::new(),
            chan2: SquareChannel::new(),
        }
    }

    pub fn get_next_event_cycle(&self) -> u64 {
        min(self.next_sample_clock, self.next_len_clock)
    }

    pub fn pump_cycle(&mut self, cpu_cycle: u64) {
        if cpu_cycle >= self.next_sample_clock {
            let value_1 = self.chan1.sample(cpu_cycle);
            let value_2 = self.chan1.sample(cpu_cycle);

            let mut value_l = 0.;
            let mut value_r = 0.;

            if self.nr51 & 0b1 != 0 {
                value_l += value_1;
            }
            if self.nr51 & 0b0001_0000 != 0 {
                value_r += value_1;
            }

            if self.nr51 & 0b10 != 0 {
                value_l += value_2;
            }
            if self.nr51 & 0b0010_0000 != 0 {
                value_r += value_2;
            }

            self.sink.emit_sample((value_l / 2., value_r / 2.));

            self.next_sample_clock += CLOCK_RATE / self.sink_rate;
            self.audio_cycle += 1;
        }

        if cpu_cycle >= self.next_len_clock {
            self.chan1.decrement_length();
            self.chan2.decrement_length();

            self.next_len_clock += CLOCK_RATE / 256;
        }

        if cpu_cycle >= self.next_env_clock {
            self.chan1.volume_env_update();
            self.chan2.volume_env_update();

            self.next_env_clock += CLOCK_RATE / 64;
        }

        if cpu_cycle >= self.next_freq_clock {
            self.chan1.freq_sweep_update();

            self.next_env_clock += CLOCK_RATE / 128;
        }
    }
}

impl MemDevice for Audio {
    fn read(&self, a: Address) -> Result<u8, ()> {
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
                REG_NR52 => Ok(self.nr52),
                _ => {
                    error!("Unimplemented sound register {:?}", a);
                    Err(())
                }
            }
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if a.in_(RNG_SND_WAV_RAM) {
            self.wav.write(a - RNG_SND_WAV_RAM.0, v)
        } else {
            match a {
                REG_NR10 => {
                    self.nr10 = v;
                    self.chan1
                        .set_freqeuncy_sweepers(v >> 4 & 0b111, v & 0b111, v & 0b1000 != 0);
                    Ok(())
                }
                REG_NR11 => {
                    self.nr11 = v;
                    self.chan1.set_duty_cycle(v >> 6);
                    self.chan1.update_length(v & 0b111111);
                    Ok(())
                }
                REG_NR12 => {
                    self.nr12 = v;
                    self.chan1.set_vol_env_period(v & 0b111);
                    self.chan1.increment_vol_env(v & 0b1000 != 0);
                    self.chan1.set_volume(v >> 4);
                    Ok(())
                }
                REG_NR13 => {
                    self.nr13 = v;
                    self.chan1.set_frequency_from_bits(self.nr14, self.nr13);
                    Ok(())
                }
                REG_NR14 => {
                    self.nr14 = v;
                    self.chan1.set_frequency_from_bits(self.nr14, self.nr13);
                    self.chan1.use_length_counter(v & 0b0100_0000 != 0);
                    Ok(())
                }
                REG_NR21 => {
                    self.nr21 = v;
                    self.chan2.set_duty_cycle(v >> 6);
                    self.chan2.update_length(v & 0b111111);
                    Ok(())
                }
                REG_NR22 => {
                    self.nr22 = v;
                    self.chan2.set_vol_env_period(v & 0b111);
                    self.chan2.increment_vol_env(v & 0b1000 != 0);
                    self.chan2.set_volume(v >> 4);
                    Ok(())
                }
                REG_NR23 => {
                    self.nr23 = v;
                    self.chan2.set_frequency_from_bits(self.nr24, self.nr23);
                    Ok(())
                }
                REG_NR24 => {
                    self.nr24 = v;
                    self.chan2.set_frequency_from_bits(self.nr24, self.nr23);
                    self.chan2.use_length_counter(v & 0b0100_0000 != 0);
                    Ok(())
                }
                REG_NR30 => {
                    self.nr30 = v;
                    Ok(())
                }
                REG_NR31 => {
                    self.nr31 = v;
                    Ok(())
                }
                REG_NR32 => {
                    self.nr32 = v;
                    Ok(())
                }
                REG_NR33 => {
                    self.nr33 = v;
                    Ok(())
                }
                REG_NR34 => {
                    self.nr34 = v;
                    Ok(())
                }
                REG_NR41 => {
                    self.nr41 = v;
                    Ok(())
                }
                REG_NR42 => {
                    self.nr42 = v;
                    Ok(())
                }
                REG_NR43 => {
                    self.nr43 = v;
                    Ok(())
                }
                REG_NR44 => {
                    self.nr44 = v;
                    Ok(())
                }
                REG_NR50 => {
                    self.nr50 = v;
                    Ok(())
                }
                REG_NR51 => {
                    self.nr51 = v;
                    Ok(())
                }
                REG_NR52 => {
                    self.nr52 = v;
                    Ok(())
                }
                _ => {
                    error!("Unimplemented sound register {:?}", a);
                    Err(())
                }
            }
        }
    }
}

struct SquareChannel {
    period: u64,
    duty_cycle: u8,
    use_len: bool,
    len: u8,

    vol: u8,
    vol_env_period: u8,
    vol_env_counter: u8,
    vol_env_increment: bool,

    frequency: u64,
    frequency_period: u8,
    frequency_counter: u8,
    frequency_shift: u8,
    frequency_increment: bool,
}

const DUTY_VALUES: [[f32; 8]; 4] = [
    [-1., -1., -1., -1., -1., -1., -1., 1.],
    [1., -1., -1., -1., -1., -1., -1., 1.],
    [1., -1., -1., -1., -1., 1., 1., 1.],
    [-1., 1., 1., 1., 1., 1., 1., -1.],
];

impl SquareChannel {
    fn new() -> SquareChannel {
        SquareChannel {
            period: 0,
            duty_cycle: 0,
            use_len: false,
            len: 0,

            vol: 0,
            vol_env_period: 0,
            vol_env_counter: 0,
            vol_env_increment: false,

            frequency: 0,
            frequency_period: 0,
            frequency_counter: 0,
            frequency_shift: 0,
            frequency_increment: false,
        }
    }

    fn set_volume(&mut self, vol: u8) {
        self.vol = vol;
    }

    fn set_vol_env_period(&mut self, p: u8) {
        self.vol_env_period = p;
    }

    fn increment_vol_env(&mut self, inc: bool) {
        self.vol_env_increment = inc;
    }

    fn set_freqeuncy_sweepers(
        &mut self,
        freqeuncy_period: u8,
        freqeuncy_shift: u8,
        freqeuncy_increment: bool,
    ) {
        self.frequency_period = freqeuncy_period;
        self.frequency_shift = freqeuncy_shift;
        self.frequency_increment = freqeuncy_increment;
    }

    fn freq_sweep_update(&mut self) {
        if self.frequency_period == 0 {
            return;
        }

        self.frequency_counter += 1;
        if self.frequency_counter >= self.frequency_period {
            let operand = self.frequency >> self.frequency_shift;
            let mut new_f = self.frequency;
            if self.frequency_increment {
                new_f += operand;
                if new_f > 2049 {
                    new_f = 2049;
                }
            } else {
                if self.frequency_shift != 0 && new_f >= operand {
                    new_f -= operand;
                }
            }
            self.frequency = new_f;
            self.update_from_frequency();
            self.frequency_counter = 0;
        }
    }

    fn set_frequency_from_bits(&mut self, hi: u8, lo: u8) {
        let x = (0b111 & (hi as u64) << 8) | lo as u64;
        self.frequency = x;
        self.update_from_frequency();
    }

    fn update_from_frequency(&mut self) {
        if self.frequency <= 2048 {
            let f = 131072 / (2048 - self.frequency);
            self.period = CLOCK_RATE / f;
            //self.period = (2048 - self.frequency) * 4;
        }
    }

    fn set_duty_cycle(&mut self, duty_cycle: u8) {
        self.duty_cycle = duty_cycle;
    }

    fn decrement_length(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

    fn update_length(&mut self, len: u8) {
        self.len = len;
    }

    fn use_length_counter(&mut self, use_len: bool) {
        self.use_len = use_len;
    }

    fn volume_env_update(&mut self) {
        if self.vol_env_period == 0 {
            return;
        }

        self.vol_env_counter += 1;
        if self.vol_env_counter >= self.vol_env_period {
            if self.vol_env_increment && self.vol < 15 {
                self.vol += 1;
            } else if self.vol != 0 {
                self.vol -= 1;
            }
            self.vol_env_counter = 0;
        }
    }

    fn sample(&mut self, cpu_cycle: u64) -> f32 {
        if self.period == 0 || self.frequency > 2048 {
            return 0.;
        }
        let phase = cpu_cycle % self.period;

        let mut duty_cycle_step = phase / (self.period / 8);
        if duty_cycle_step > 7 {
            duty_cycle_step = 7;
        }

        DUTY_VALUES[self.duty_cycle as usize][duty_cycle_step as usize] * (self.vol as f32 / 15.0)
    }
}
