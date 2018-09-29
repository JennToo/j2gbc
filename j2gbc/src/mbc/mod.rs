pub mod mbc1;

use super::mem::{Address, ExtendedAddress, MemDevice};

pub trait Mbc: MemDevice {
    fn map_address_into_rom(&self, a: Address) -> ExtendedAddress;

    fn get_sram(&self) -> &[u8];
    fn set_sram(&mut self, buf: &[u8]);
}
