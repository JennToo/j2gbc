pub struct Mixer {
    left_enable: [bool; 4],
    right_enable: [bool; 4],

    left_master_vol: f32,
    right_master_vol: f32,
}

impl Mixer {
    pub fn new() -> Mixer {
        Mixer {
            left_enable: [false; 4],
            right_enable: [false; 4],

            left_master_vol: 0.,
            right_master_vol: 0.,
        }
    }

    pub fn mix(&self, samples: [f32; 4]) -> (f32, f32) {
        let mut left_val = 0.;
        for i in 0..4 {
            if self.left_enable[i] {
                left_val += samples[i];
            }
        }
        left_val /= 4.;

        let mut right_val = 0.;
        for i in 0..4 {
            if self.right_enable[i] {
                right_val += samples[i];
            }
        }
        right_val /= 4.;

        (
            left_val * self.left_master_vol,
            right_val * self.right_master_vol,
        )
    }

    pub fn set_enabled_channels(&mut self, left_enable: [bool; 4], right_enable: [bool; 4]) {
        self.left_enable = left_enable;
        self.right_enable = right_enable;
    }

    pub fn set_master_volumes(&mut self, left: f32, right: f32) {
        self.left_master_vol = left;
        self.right_master_vol = right;
    }
}
