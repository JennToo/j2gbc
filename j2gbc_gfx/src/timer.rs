use std::time::{Duration, Instant};

pub struct DeltaTimer {
    last_time: Instant,
}

impl DeltaTimer {
    pub fn new() -> DeltaTimer {
        DeltaTimer {
            last_time: Instant::now(),
        }
    }

    pub fn elapsed(&mut self) -> Duration {
        let new_now = Instant::now();
        let d = new_now - self.last_time;
        self.last_time = new_now;
        d
    }
}
