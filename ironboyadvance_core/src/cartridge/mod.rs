use std::path::PathBuf;

use header::Header;
use ironboyadvance_arm7tdmi::memory::IoMemoryAccess;
use ironboyadvance_utils::read_file;

use crate::{
    GbaError,
    system_bus::{ROM_WS0_HI, ROM_WS0_LO, ROM_WS1_HI, ROM_WS1_LO, ROM_WS2_HI, ROM_WS2_LO, SRAM_HI, SRAM_LO},
};

pub mod header;

const MAX_CARTRIDGE_BYTES: usize = 32 * 1024 * 1024;

pub struct Cartridge {
    header: Header,
    data: Vec<u8>,
}

impl Cartridge {
    pub fn load(path: PathBuf) -> Result<Cartridge, GbaError> {
        let buffer = match read_file(&path) {
            Ok(buffer) => buffer.into_boxed_slice(),
            Err(_) => return Err(GbaError::FileLoadFailure),
        };

        let header = Header::load(&buffer[0..228])?;
        println!("{}", header.game_title());
        println!("{}", header.game_code());

        let mut data = vec![0; MAX_CARTRIDGE_BYTES];
        data[..buffer.len()].clone_from_slice(&buffer);

        Ok(Cartridge { header, data })
    }
}

impl IoMemoryAccess for Cartridge {
    fn read_8(&self, address: u32) -> u8 {
        match address & 0xFF000000 {
            ROM_WS0_LO | ROM_WS0_HI => self.data[(address - ROM_WS0_LO) as usize],
            ROM_WS1_LO | ROM_WS1_HI => self.data[(address - ROM_WS1_LO) as usize],
            ROM_WS2_LO | ROM_WS2_HI => self.data[(address - ROM_WS2_LO) as usize],
            SRAM_LO | SRAM_HI => self.data[(address - SRAM_LO) as usize],
            _ => panic!("Read to address {:08X} invalid", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address & 0xFF000000 {
            ROM_WS0_LO | ROM_WS0_HI => self.data[(address - ROM_WS0_LO) as usize] = value,
            ROM_WS1_LO | ROM_WS1_HI => self.data[(address - ROM_WS1_LO) as usize] = value,
            ROM_WS2_LO | ROM_WS2_HI => self.data[(address - ROM_WS2_LO) as usize] = value,
            SRAM_LO | SRAM_HI => self.data[(address - SRAM_LO) as usize] = value,
            _ => panic!("Write to address {:08X} invalid", address),
        }
    }
}
