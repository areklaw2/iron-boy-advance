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

    fn in_range(&self, address: u32) -> bool {
        (0x0E000000..=0x0FFFFFFF).contains(&address)
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

    fn read_16(&self, address: u32) -> u16 {
        match self.in_range(address) {
            true => u16::from_le_bytes([self.read_8(address); 2]),
            false => self.rom_read(address) as u16 | (self.rom_read(address + 1) as u16) << 8,
        }
    }

    fn read_32(&self, address: u32) -> u32 {
        match self.in_range(address) {
            true => u32::from_le_bytes([self.read_8(address); 4]),
            false => self.read_16(address) as u32 | (self.read_16(address + 2) as u32) << 16,
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            0x08000000..=0x0DFFFFFF => {}
            0x0E000000..=0x0FFFFFFF => self.backup_file.write((address & 0x7FFF) as usize, value),
            _ => panic!("Invalid byte write for Sram: {:08X}", address),
        }
    }

    fn write_16(&mut self, address: u32, value: u16) {
        if self.in_range(address) {
            self.write_8(address, (value >> ((address & 1) * 8)) as u8);
        }
    }

    fn write_32(&mut self, address: u32, value: u32) {
        if self.in_range(address) {
            self.write_8(address, (value >> ((address & 3) * 8)) as u8);
        }
    }
}

impl CartridgeBackup for Sram {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
