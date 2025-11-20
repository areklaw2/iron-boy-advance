use std::path::PathBuf;

use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use ironboyadvance_utils::read_file;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BiosError {
    #[error("Bios read failed")]
    ReadFailure,
}

pub struct Bios {
    data: Box<[u8]>,
}

impl Bios {
    pub fn load(path: PathBuf) -> Result<Bios, BiosError> {
        let buffer = match read_file(&path) {
            Ok(buffer) => buffer.into_boxed_slice(),
            Err(_) => return Err(BiosError::ReadFailure),
        };
        Ok(Bios { data: buffer })
    }
}

impl SystemMemoryAccess for Bios {
    fn read_8(&self, address: u32) -> u8 {
        self.data[address as usize]
    }

    fn write_8(&mut self, _address: u32, _value: u8) {}
}
