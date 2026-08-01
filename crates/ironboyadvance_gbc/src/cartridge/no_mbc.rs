use std::path::Path;

use ironboyadvance_common::memory::SystemMemoryAccess;

use super::{CartridgeError, MemoryBankController, backup_file::BackupFile};

const RAM_BANK_SIZE: usize = 0x2000;

pub struct NoMbc {
    rom: Vec<u8>,
    ram: BackupFile,
}

impl NoMbc {
    pub fn new(buffer: Vec<u8>, has_ram: bool, has_battery: bool, save_file: &Path) -> Result<NoMbc, CartridgeError> {
        let ram_size = match has_ram {
            true => RAM_BANK_SIZE,
            false => 0,
        };

        let ram = match has_battery {
            true => BackupFile::open(save_file, ram_size, 0x00)?,
            false => BackupFile::memory(ram_size, 0x00),
        };

        Ok(NoMbc { rom: buffer, ram })
    }
}

impl SystemMemoryAccess for NoMbc {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => *self.rom.get(address as usize).unwrap_or(&0xFF),
            0xA000..=0xBFFF => match self.ram.len() {
                0 => 0xFF,
                _ => self.ram.read(address as usize & (RAM_BANK_SIZE - 1)),
            },
            _ => panic!("Invalid byte read for NoMbc: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {}
            0xA000..=0xBFFF => match self.ram.len() {
                0 => {}
                _ => self.ram.write(address as usize & (RAM_BANK_SIZE - 1), value),
            },
            _ => panic!("Invalid byte write for NoMbc: {:#06X}", address),
        }
    }
}

impl MemoryBankController for NoMbc {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
