use std::path::Path;

use ironboyadvance_common::memory::SystemMemoryAccess;

use crate::cartridge::{CartridgeBackup, CartridgeError, backup_file::BackupFile};

pub const EEPROM_512B_SIZE: usize = 0x200;
pub const EEPROM_8KB_SIZE: usize = 0x2000;

pub struct Eeprom {
    rom: Vec<u8>,
    backup_file: BackupFile,
}

impl Eeprom {
    pub fn new(rom: Vec<u8>, save_file: &Path) -> Result<Self, CartridgeError> {
        let backup_file = BackupFile::open(save_file, EEPROM_512B_SIZE, 0x00)?;
        Ok(Self { rom, backup_file })
    }
}

impl SystemMemoryAccess for Eeprom {
    fn read_8(&self, address: u32) -> u8 {
        if self.rom.len() > 0x1000000 {
            match address {
                0x0DFFFF00..=0x0DFFFFFF => self.rom_read(address),
                _ => panic!("Invalid byte read for Eeprom: {:08X}", address),
            }
        } else {
            match address {
                0x0D000000..=0x0DFFFFFF => self.rom_read(address),
                _ => panic!("Invalid byte read for Eeprom: {:08X}", address),
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        if self.rom.len() > 0x1000000 {
            match address {
                0x0DFFFF00..=0x0DFFFFFF => {}
                _ => panic!("Invalid byte write for Eeprom: {:08X}", address),
            }
        } else {
            match address {
                0x0D000000..=0x0DFFFFFF => {}
                _ => panic!("Invalid byte write for Eeprom: {:08X}", address),
            }
        }
    }
}

impl CartridgeBackup for Eeprom {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
