use toml::Value;

use crate::mem::{Address, AddressRange};

#[derive(Default)]
pub struct MmuExceptions {
    ranges: Vec<AddressRange>,
}

const EXCEPTIONS: &str = include_str!("../mmu_exceptions.toml");

impl MmuExceptions {
    pub fn from_title(title: &str) -> MmuExceptions {
        let doc = EXCEPTIONS.parse::<Value>().unwrap();

        if let Some(v) = doc.get(title) {
            if let Some(ranges) = v.get("ranges") {
                if let Some(ranges_arr) = ranges.as_array() {
                    let mut range_vec = vec![];
                    for range in ranges_arr {
                        if let Some(range_arr) = range.as_array() {
                            if range_arr.len() == 2 {
                                let first = u16::from_str_radix(range_arr[0].as_str().unwrap(), 16)
                                    .unwrap();
                                let second =
                                    u16::from_str_radix(range_arr[1].as_str().unwrap(), 16)
                                        .unwrap();

                                range_vec.push(AddressRange(Address(first), Address(second)));
                            }
                        }
                    }

                    return MmuExceptions { ranges: range_vec };
                }
            }
        }

        MmuExceptions::default()
    }

    pub fn allow(&self, a: Address) -> bool {
        for r in &self.ranges {
            if a.in_(*r) {
                return true;
            }
        }

        return false;
    }
}
