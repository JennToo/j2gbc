use j2ds::{next_timer_event, Timer, TimerEvent};

use super::{
    mixer::Mixer, noise::NoiseChannel, square::SquareChannel, wave::WaveChannel, AudioSink,
};
use crate::cpu::CLOCK_RATE;

pub struct Synth {
    sink: Box<AudioSink>,

    sample_clock: Timer,
    len_clock: Timer,
    env_clock: Timer,
    freq_clock: Timer,

    pub mixer: Mixer,

    pub chan1: SquareChannel,
    pub chan2: SquareChannel,
    pub chan3: WaveChannel,
    pub chan4: NoiseChannel,
}

impl Synth {
    pub fn new(sink: Box<AudioSink>) -> Synth {
        Synth {
            sample_clock: Timer::new(CLOCK_RATE / sink.sample_rate(), 0, 0),
            len_clock: Timer::new(CLOCK_RATE / 256, 0, 0),
            env_clock: Timer::new(CLOCK_RATE / 64, 0, 0),
            freq_clock: Timer::new(CLOCK_RATE / 128, 0, 0),

            sink,

            mixer: Mixer::new(),

            chan1: SquareChannel::new(),
            chan2: SquareChannel::new(),
            chan3: WaveChannel::new(),
            chan4: NoiseChannel::new(),
        }
    }

    pub fn get_next_event_cycle(&self) -> u64 {
        next_timer_event(&[
            self.sample_clock,
            self.len_clock,
            self.env_clock,
            self.freq_clock,
        ])
    }

    pub fn pump_cycle(&mut self, cpu_cycle: u64) {
        if self.sample_clock.update(cpu_cycle) == Some(TimerEvent::RisingEdge) {
            let samples = [
                self.chan1.sample(cpu_cycle),
                self.chan2.sample(cpu_cycle),
                self.chan3.sample(cpu_cycle),
                self.chan4.sample(cpu_cycle),
            ];
            self.sink.emit_sample(self.mixer.mix(samples));
        }

        if self.len_clock.update(cpu_cycle) == Some(TimerEvent::RisingEdge) {
            self.chan1.decrement_length();
            self.chan2.decrement_length();
            self.chan3.decrement_length();
            self.chan4.decrement_length();
        }

        if self.env_clock.update(cpu_cycle) == Some(TimerEvent::RisingEdge) {
            self.chan1.volume_env_update();
            self.chan2.volume_env_update();
            self.chan4.volume_env_update();
        }

        if self.freq_clock.update(cpu_cycle) == Some(TimerEvent::RisingEdge) {
            self.chan1.freq_sweep_update();
        }
    }
}
