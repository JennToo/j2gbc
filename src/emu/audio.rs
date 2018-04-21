use super::mem::{Address, MemDevice, Ram, RNG_SND_WAV_RAM};

const REG_NR50: Address = Address(0xFF24);
const REG_NR51: Address = Address(0xFF25);
const REG_NR52: Address = Address(0xFF26);

pub struct Audio {
    wav: Ram,
    nr50: u8,
    nr51: u8,
    nr52: u8,
}

impl Audio {
    pub fn new() -> Audio {
        Audio {
            wav: Ram::new(RNG_SND_WAV_RAM.len()),
            nr50: 0,
            nr51: 0,
            nr52: 0,
        }
    }
}

impl MemDevice for Audio {
    fn read(&self, a: Address) -> Result<u8, ()> {
        if a.in_(RNG_SND_WAV_RAM) {
            self.wav.read(a - RNG_SND_WAV_RAM.0)
        } else if a == REG_NR50 {
            Ok(self.nr50)
        } else if a == REG_NR51 {
            Ok(self.nr51)
        } else if a == REG_NR52 {
            Ok(self.nr52)
        } else {
            println!("Unimplemented sound register {:?}", a);
            Err(())
        }
    }

    fn write(&mut self, a: Address, v: u8) -> Result<(), ()> {
        if a.in_(RNG_SND_WAV_RAM) {
            self.wav.write(a - RNG_SND_WAV_RAM.0, v)
        } else if a == REG_NR50 {
            self.nr50 = v;
            Ok(())
        } else if a == REG_NR51 {
            self.nr51 = v;
            Ok(())
        } else if a == REG_NR52 {
            self.nr52 = v;
            Ok(())
        } else {
            println!("Unimplemented sound register {:?}", a);
            Err(())
        }
    }
}
