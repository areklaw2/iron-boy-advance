use std::path::PathBuf;

use utils::read_file;

use crate::{memory::IoMemoryAccess, GbaError};

pub struct Bios {
    data: Box<[u8]>,
}

impl Bios {
    pub fn load(path: PathBuf) -> Result<Bios, GbaError> {
        let buffer = match read_file(&path) {
            Ok(buffer) => buffer.into_boxed_slice(),
            Err(_) => return Err(GbaError::FileLoadFailure),
        };
        Ok(Bios { data: buffer })
    }
}

impl IoMemoryAccess for Bios {
    fn read_8(&self, address: u32, _is_instruction: bool) -> u8 {
        self.data[address as usize]
    }

    fn write_8(&mut self, address: u32, value: u8) {
        self.data[address as usize] = value
    }
}
