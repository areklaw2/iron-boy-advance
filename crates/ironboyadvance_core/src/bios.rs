use std::cell::Cell;

use getset::CopyGetters;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use thiserror::Error;

const BIOS_END: u32 = 0x3FFF;

#[derive(Error, Debug)]
pub enum BiosError {
    #[error("Invalid bios length: {0}")]
    InvalidBiosLength(usize),
}

#[derive(Debug, CopyGetters)]
pub struct Bios {
    data: Box<[u8]>,
    #[getset(get_copy = "pub")]
    loaded: bool,
    last_fetched: Cell<u32>,
    pc_in_bios: bool,
}

impl Bios {
    pub fn load(buffer: Box<[u8]>) -> Result<Bios, BiosError> {
        let length = buffer.len();
        if length != 0x4000 {
            return Err(BiosError::InvalidBiosLength(length));
        }

        let (data, loaded) = match !buffer.is_empty() {
            true => (buffer, true),
            false => (Box::new([0u8; 0x4000]) as Box<[u8]>, false),
        };
        Ok(Self {
            data,
            loaded,
            last_fetched: Cell::new(0xE129F000),
            pc_in_bios: true,
        })
    }

    pub fn set_pc_ref(&mut self, pc: u32) {
        self.pc_in_bios = pc <= BIOS_END;
    }
}

impl SystemMemoryAccess for Bios {
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
        if self.pc_in_bios && aligned_address <= BIOS_END {
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
