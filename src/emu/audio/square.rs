use j2ds::Clock;

pub struct SquareChannel {
    period: u64,
    duty_cycle: u8,
    use_len: bool,
    len: u8,

    vol: u8,
    vol_orig: u8,
    vol_env_increment: bool,
    vol_counter: Clock,

    frequency: u64,
    frequency_shift: u8,
    frequency_increment: bool,
    frequency_sweep_counter: Clock,
}

const DUTY_VALUES: [[f32; 8]; 4] = [
    [-1., -1., -1., -1., -1., -1., -1., 1.],
    [1., -1., -1., -1., -1., -1., -1., 1.],
    [1., -1., -1., -1., -1., 1., 1., 1.],
    [-1., 1., 1., 1., 1., 1., 1., -1.],
];

impl SquareChannel {
    pub fn new() -> SquareChannel {
        SquareChannel {
            period: 0,
            duty_cycle: 0,
            use_len: false,
            len: 0,

            vol: 0,
            vol_orig: 0,
            vol_env_increment: false,
            vol_counter: Clock::new(0),

            frequency: 0,
            frequency_shift: 0,
            frequency_increment: false,
            frequency_sweep_counter: Clock::new(0),
        }
    }

    pub fn reset(&mut self) {
        self.frequency_sweep_counter.reset();
        self.vol_counter.reset();
        if self.len == 0 {
            self.len = 64;
        }
        self.vol = self.vol_orig;
    }

    pub fn set_volume(&mut self, vol: u8) {
        self.vol = vol;
        self.vol_orig = vol;
    }

    pub fn set_vol_env_period(&mut self, p: u8) {
        self.vol_counter = Clock::new(p as u64);
    }

    pub fn increment_vol_env(&mut self, inc: bool) {
        self.vol_env_increment = inc;
    }

    pub fn set_freqeuncy_sweepers(
        &mut self,
        freqeuncy_period: u8,
        freqeuncy_shift: u8,
        freqeuncy_increment: bool,
    ) {
        self.frequency_sweep_counter = Clock::new(freqeuncy_period as u64);
        self.frequency_shift = freqeuncy_shift;
        self.frequency_increment = freqeuncy_increment;
    }

    pub fn freq_sweep_update(&mut self) {
        if self.frequency_sweep_counter.period() == 0 {
            return;
        }

        if self.frequency_sweep_counter.tick() {
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
        }
    }

    pub fn set_frequency_from_bits(&mut self, hi: u8, lo: u8) {
        self.frequency = (hi as u64 & 0b111) << 8 | lo as u64;
        self.update_from_frequency();
    }

    fn update_from_frequency(&mut self) {
        if self.frequency <= 2048 {
            self.period = 4 * 8 * (2048 - self.frequency);
        }
    }

    pub fn set_duty_cycle(&mut self, duty_cycle: u8) {
        self.duty_cycle = duty_cycle;
    }

    pub fn decrement_length(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

    pub fn update_length(&mut self, len: u8) {
        self.len = len;
    }

    pub fn use_length_counter(&mut self, use_len: bool) {
        self.use_len = use_len;
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

    pub fn sample(&self, cpu_cycle: u64) -> f32 {
        if self.period == 0 || self.frequency > 2048 || !self.is_active() {
            return 0.;
        }
        let phase = cpu_cycle % self.period;

        let duty_cycle_step = phase * 8 / self.period;

        DUTY_VALUES[self.duty_cycle as usize][duty_cycle_step as usize] * (self.vol as f32 / 15.0)
    }

    pub fn is_active(&self) -> bool {
        !self.use_len || self.len > 0
    }
}
