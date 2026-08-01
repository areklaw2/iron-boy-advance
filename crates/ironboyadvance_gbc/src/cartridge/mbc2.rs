use std::path::Path;

use ironboyadvance_common::memory::SystemMemoryAccess;

use super::{CartridgeError, MemoryBankController, backup_file::BackupFile};

const BUILT_IN_RAM_SIZE: usize = 512;
const UNUSED_RAM_BITS: u8 = 0xF0;

pub struct Mbc2 {
    rom: Vec<u8>,
    ram: BackupFile,
    ram_enabled: bool,
    current_rom_bank: usize,
    rom_banks: usize,
}

impl Mbc2 {
    pub fn new(buffer: Vec<u8>, rom_banks: usize, has_battery: bool, save_file: &Path) -> Result<Mbc2, CartridgeError> {
        let ram = match has_battery {
            true => BackupFile::open(save_file, BUILT_IN_RAM_SIZE, UNUSED_RAM_BITS)?,
            false => BackupFile::memory(BUILT_IN_RAM_SIZE, UNUSED_RAM_BITS),
        };

        Ok(Mbc2 {
            rom: buffer,
            ram,
            ram_enabled: false,
            current_rom_bank: 1,
            rom_banks,
        })
    }
}

impl SystemMemoryAccess for Mbc2 {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom_read(0, address),
            0x4000..=0x7FFF => self.rom_read(self.current_rom_bank, address),
            0xA000..=0xBFFF => match self.ram_enabled {
                true => self.ram.read(address as usize & (BUILT_IN_RAM_SIZE - 1)) | UNUSED_RAM_BITS,
                false => 0xFF,
            },
            _ => panic!("Invalid byte read for Mbc2: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => match address & 0x100 == 0 {
                true => self.ram_enabled = value & 0xF == 0xA,
                false => {
                    self.current_rom_bank = match (value as usize) & 0x0F {
                        0 => 1,
                        n => n,
                    } % self.rom_banks
                }
            },
            0x4000..=0x7FFF => {}
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    self.ram
                        .write(address as usize & (BUILT_IN_RAM_SIZE - 1), value | UNUSED_RAM_BITS);
                }
            }
            _ => panic!("Invalid byte write for Mbc2: {:#06X}", address),
        }
    }
}

impl MemoryBankController for Mbc2 {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
