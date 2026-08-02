use std::path::Path;

use ironboyadvance_common::memory::SystemMemoryAccess;

use crate::cartridge::{CartridgeBackup, CartridgeError, backup_file::BackupFile};

const SRAM_SIZE: usize = 32 * 1024;

pub struct Sram {
    rom: Vec<u8>,
    backup_file: BackupFile,
}

impl Sram {
    pub fn new(rom: Vec<u8>, save_file: &Path) -> Result<Self, CartridgeError> {
        let backup_file = BackupFile::open(save_file, SRAM_SIZE, 0xFF)?;
        Ok(Self { rom, backup_file })
    }
}

impl SystemMemoryAccess for Sram {
    type Address = u32;

    fn read_8(&self, address: u32) -> u8 {
        match address {
            0x08000000..=0x0DFFFFFF => self.rom_read(address),
            0x0E000000..=0x0FFFFFFF => self.backup_file.read((address & 0x7FFF) as usize),
            _ => panic!("Invalid byte read for Sram: {:08X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            0x08000000..=0x0DFFFFFF => {}
            0x0E000000..=0x0FFFFFFF => self.backup_file.write((address & 0x7FFF) as usize, value),
            _ => panic!("Invalid byte write for Sram: {:08X}", address),
        }
    }
}

impl CartridgeBackup for Sram {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
