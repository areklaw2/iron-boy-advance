use getset::CopyGetters;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use std::path::PathBuf;
use thiserror::Error;

use crate::read_file;

#[derive(Error, Debug)]
pub enum BiosError {
    #[error("Bios read failed")]
    ReadFailure,
}

#[derive(Debug, CopyGetters)]
pub struct Bios {
    data: Box<[u8]>,
    #[getset(get_copy = "pub")]
    loaded: bool,
}

impl Bios {
    pub fn load(path: Option<PathBuf>) -> Result<Bios, BiosError> {
        let (data, loaded) = match path {
            Some(path) => {
                let buffer = match read_file(&path) {
                    Ok(buffer) => buffer.into_boxed_slice(),
                    Err(_) => return Err(BiosError::ReadFailure),
                };
                (buffer, true)
            }
            None => (Box::new([0u8; 0x4000]) as Box<[u8]>, false),
        };
        Ok(Self { data, loaded })
    }
}

impl SystemMemoryAccess for Bios {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            0..=0x3FFF => self.data[address as usize],
            _ => panic!("Invalid byte read for Bios: {:08X}", address),
        }
    }

    fn write_8(&mut self, _address: u32, _value: u8) {}
}
