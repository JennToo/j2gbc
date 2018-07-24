pub struct Counter {
    pub count: u64,
    pub period: u64,
}

impl Counter {
    pub fn new(period: u64) -> Counter {
        Counter {
            count: 0,
            period: period,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.count += 1;
        if self.count >= self.period {
            self.count = 0;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }
}

#[test]
fn test_counter() {
    let mut c = Counter::new(3);

    // First round
    assert!(!c.tick());
    assert!(!c.tick());
    assert!(c.tick());

    // Wrap and second round
    assert!(!c.tick());
    assert!(!c.tick());
    assert!(c.tick());

    // Reset in the middle of a cycle
    assert!(!c.tick());
    c.reset();
    assert!(!c.tick());
    assert!(!c.tick());
    assert!(c.tick());
}
