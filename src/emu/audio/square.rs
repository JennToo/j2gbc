use emu::cpu::CLOCK_RATE;

pub struct SquareChannel {
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
    pub fn new() -> SquareChannel {
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

    pub fn set_volume(&mut self, vol: u8) {
        self.vol = vol;
    }

    pub fn set_vol_env_period(&mut self, p: u8) {
        self.vol_env_period = p;
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
        self.frequency_period = freqeuncy_period;
        self.frequency_shift = freqeuncy_shift;
        self.frequency_increment = freqeuncy_increment;
    }

    pub fn freq_sweep_update(&mut self) {
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

    pub fn set_frequency_from_bits(&mut self, hi: u8, lo: u8) {
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

    pub fn sample(&mut self, cpu_cycle: u64) -> f32 {
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
