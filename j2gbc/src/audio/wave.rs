pub struct WaveChannel {
    samples: [f32; 32],
    period: u64,
    pub use_len: bool,
    len: u8,

    pub enabled: bool,

    pub vol_multiplier: f32,

    position_offset_cycle: u64,
    last_cpu_cycle: u64,
}

impl WaveChannel {
    pub fn new() -> WaveChannel {
        WaveChannel {
            samples: [-1.; 32],
            period: 0,
            use_len: false,
            len: 0,

            enabled: false,
            vol_multiplier: 0.,

            position_offset_cycle: 0,
            last_cpu_cycle: 0,
        }
    }

    pub fn set_frequency_from_bits(&mut self, hi: u8, lo: u8) {
        let f = (u64::from(hi) & 0b111) << 8 | u64::from(lo);
        self.period = (2048 - f) * 2 * 32;
    }

    pub fn set_len(&mut self, len: u8) {
        self.len = len;
    }

    pub fn decrement_length(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

    pub fn write_sample(&mut self, sample: f32, position: usize) {
        self.samples[position] = sample;
    }

    pub fn sample(&mut self, cpu_cycle: u64) -> f32 {
        if self.period == 0 || !self.is_active() {
            return 0.;
        }
        self.last_cpu_cycle = cpu_cycle;

        let phase = (cpu_cycle + self.position_offset_cycle) % self.period;
        let position = phase * 32 / self.period;
        self.samples[position as usize] * self.vol_multiplier
    }

    pub fn is_active(&self) -> bool {
        (!self.use_len || self.len > 0) && self.enabled
    }

    pub fn reset(&mut self) {
        self.position_offset_cycle = self.last_cpu_cycle;
        if self.len == 0 {
            self.len = 255;
        }
    }
}
