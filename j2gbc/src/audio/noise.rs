use j2ds::Clock;

pub struct NoiseChannel {
    lfsr: u16,
    lfsr_half: bool,

    period: u64,

    len: u8,
    pub use_len: bool,

    next_lfsr_shift_cycle: u64,
    last_cpu_cycle: u64,

    vol: u8,
    vol_orig: u8,
    vol_env_increment: bool,
    vol_counter: Clock,
}

const DIVISORS: [u64; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

impl NoiseChannel {
    pub fn new() -> NoiseChannel {
        NoiseChannel {
            lfsr: 0b1111_1111,
            lfsr_half: false,

            period: 0,

            len: 0,
            use_len: false,

            next_lfsr_shift_cycle: 0,
            last_cpu_cycle: 0,

            vol: 0,
            vol_orig: 0,
            vol_env_increment: false,
            vol_counter: Clock::new(0),
        }
    }

    pub fn set_frequency_from_bits(&mut self, bits: u8) {
        let s = (bits >> 4) & 0b1111;
        let r = DIVISORS[(bits & 0b111) as usize];
        self.period = r << s;

        self.lfsr_half = bits & 0b0000_1000 != 0;
    }

    pub fn set_len(&mut self, len: u8) {
        self.len = 64 - len;
    }

    pub fn decrement_length(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

    pub fn set_volume(&mut self, vol: u8) {
        self.vol = vol;
        self.vol_orig = vol;
    }

    pub fn set_vol_env_period(&mut self, p: u8) {
        self.vol_counter = Clock::new(u64::from(p));
    }

    pub fn increment_vol_env(&mut self, inc: bool) {
        self.vol_env_increment = inc;
    }

    pub fn volume_env_update(&mut self) {
        if self.vol_counter.period() == 0 {
            return;
        }

        if self.vol_counter.tick() {
            if self.vol_env_increment && self.vol < 15 {
                self.vol += 1;
            } else if self.vol != 0 {
                self.vol -= 1;
            }
        }
    }

    pub fn sample(&mut self, cpu_cycle: u64) -> f32 {
        if self.period == 0 || !self.is_active() {
            return 0.;
        }

        while self.next_lfsr_shift_cycle <= cpu_cycle {
            let shift = if self.lfsr_half { 6 } else { 14 };

            let downshifted = self.lfsr >> 1;
            let bit_1 = self.lfsr & 0b1;
            let bit_2 = downshifted & 0b1;
            let new_bit = (bit_1 ^ bit_2) << shift;
            let cleared = downshifted & !(1 << shift);

            self.lfsr = cleared | new_bit;

            self.next_lfsr_shift_cycle += self.period;
        }
        self.last_cpu_cycle = cpu_cycle;

        (f32::from(self.vol) / 15.) * if self.lfsr & 0b1 != 0 { -1. } else { 1. }
    }

    pub fn is_active(&self) -> bool {
        !self.use_len || self.len > 0
    }

    pub fn reset(&mut self) {
        if self.len == 0 {
            self.len = 64;
        }
        self.next_lfsr_shift_cycle = self.period + self.last_cpu_cycle;
        self.vol = self.vol_orig;
        self.lfsr = 0b1111_1111;
    }
}
