use std::cmp::min;

use super::mixer::Mixer;
use super::square::SquareChannel;
use super::AudioSink;
use emu::cpu::CLOCK_RATE;

pub struct Synth {
    sink: Box<AudioSink>,
    sink_rate: u64,

    next_sample_clock: u64,
    next_len_clock: u64,
    next_env_clock: u64,
    next_freq_clock: u64,

    audio_cycle: u64,

    pub mixer: Mixer,

    pub chan1: SquareChannel,
    pub chan2: SquareChannel,
}

impl Synth {
    pub fn new(sink: Box<AudioSink>) -> Synth {
        Synth {
            sink_rate: sink.sample_rate(),
            sink,

            next_sample_clock: 0,
            next_len_clock: 0,
            next_env_clock: 0,
            next_freq_clock: 0,

            audio_cycle: 0,

            mixer: Mixer::new(),

            chan1: SquareChannel::new(),
            chan2: SquareChannel::new(),
        }
    }

    pub fn get_next_event_cycle(&self) -> u64 {
        min(
            min(self.next_sample_clock, self.next_len_clock),
            self.next_freq_clock,
        )
    }

    pub fn pump_cycle(&mut self, cpu_cycle: u64) {
        if cpu_cycle >= self.next_sample_clock {
            let samples = [
                self.chan1.sample(cpu_cycle),
                self.chan1.sample(cpu_cycle),
                self.chan2.sample(cpu_cycle),
                self.chan2.sample(cpu_cycle),
            ];

            self.sink.emit_sample(self.mixer.mix(samples));

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

            self.next_freq_clock += CLOCK_RATE / 128;
        }
    }
}
