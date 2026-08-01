use std::cell::Cell;

use getset::CopyGetters;
use ironboyadvance_common::memory::SystemMemoryAccess;
use thiserror::Error;

const BIOS_END: u32 = 0x3FFF;

#[derive(Error, Debug)]
pub enum BiosError {
    #[error("Invalid bios length: {0}")]
    InvalidBiosLength(usize),
}

#[derive(Debug, CopyGetters)]
pub struct Bios {
    data: Vec<u8>,
    #[getset(get_copy = "pub")]
    loaded: bool,
    last_fetched: Cell<u32>,
    pc_in_bios: Cell<bool>,
}

impl Bios {
    pub fn load(buffer: Vec<u8>) -> Result<Bios, BiosError> {
        let (data, loaded) = match buffer.is_empty() {
            true => (vec![0; 0x4000], false),
            false => {
                if buffer.len() != 0x4000 {
                    return Err(BiosError::InvalidBiosLength(buffer.len()));
                }
                (buffer, true)
            }
        };

        Ok(Self {
            data,
            loaded,
            last_fetched: Cell::new(0xE129F000),
            pc_in_bios: Cell::new(true),
        })
    }

    pub fn set_pc(&self, pc: u32) {
        self.pc_in_bios.set(pc <= BIOS_END);
    }
}

impl SystemMemoryAccess for Bios {
    type Address = u32;

    fn read_8(&self, address: u32) -> u8 {
        let word = self.read_32(address & !3);
        (word >> ((address & 3) * 8)) as u8
    }

    fn read_16(&self, address: u32) -> u16 {
        let aligned_address = address & !1;
        let word = self.read_32(aligned_address & !3);
        (word >> ((aligned_address & 2) * 8)) as u16
    }

    fn read_32(&self, address: u32) -> u32 {
        let aligned_address = address & !3;
        if self.pc_in_bios.get() && aligned_address <= BIOS_END {
            let address = aligned_address as usize;
            let word = u32::from_le_bytes(self.data[address..address + 4].try_into().unwrap());
            self.last_fetched.set(word);
            word
        } else {
            self.last_fetched.get()
        }
    }

    fn write_8(&mut self, _address: u32, _value: u8) {}
}
