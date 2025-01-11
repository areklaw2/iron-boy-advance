use std::{fs::File, io::Read, path::PathBuf};

use header::Header;
use utils::read_file;

use crate::{
    memory::{
        system_bus::{
            ROM_WAIT_STATE_0_END, ROM_WAIT_STATE_0_START, ROM_WAIT_STATE_1_END, ROM_WAIT_STATE_1_START,
            ROM_WAIT_STATE_2_END, ROM_WAIT_STATE_2_START, SRAM_END, SRAM_START,
        },
        IoMemoryAccess,
    },
    GbaError,
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
        match address {
            ROM_WAIT_STATE_0_START..=ROM_WAIT_STATE_0_END => self.data[(address - ROM_WAIT_STATE_0_START) as usize],
            ROM_WAIT_STATE_1_START..=ROM_WAIT_STATE_1_END => self.data[(address - ROM_WAIT_STATE_1_START) as usize],
            ROM_WAIT_STATE_2_START..=ROM_WAIT_STATE_2_END => self.data[(address - ROM_WAIT_STATE_2_START) as usize],
            SRAM_START..=SRAM_END => self.data[(address - SRAM_START) as usize],
            _ => panic!("Read to address {:08X} invalid", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            ROM_WAIT_STATE_0_START..=ROM_WAIT_STATE_0_END => self.data[(address - ROM_WAIT_STATE_0_START) as usize] = value,
            ROM_WAIT_STATE_1_START..=ROM_WAIT_STATE_1_END => self.data[(address - ROM_WAIT_STATE_1_START) as usize] = value,
            ROM_WAIT_STATE_2_START..=ROM_WAIT_STATE_2_END => self.data[(address - ROM_WAIT_STATE_2_START) as usize] = value,
            SRAM_START..=SRAM_END => self.data[(address - SRAM_START) as usize] = value,
            _ => panic!("Write to address {:08X} invalid", address),
        }
    }
}
