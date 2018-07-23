pub struct Mixer {
    left_enable: [bool; 4],
    right_enable: [bool; 4],
}

impl Mixer {
    pub fn new() -> Mixer {
        Mixer {
            left_enable: [false; 4],
            right_enable: [false; 4],
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

        (left_val, right_val)
    }

    pub fn set_enabled(&mut self, left_enable: [bool; 4], right_enable: [bool; 4]) {
        self.left_enable = left_enable;
        self.right_enable = right_enable;
    }
}
