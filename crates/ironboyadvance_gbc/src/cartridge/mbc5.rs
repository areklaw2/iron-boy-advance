use std::path::Path;

use ironboyadvance_common::memory::SystemMemoryAccess;

use super::{CartridgeError, MemoryBankController, backup_file::BackupFile};

const RAM_BANK_SIZE: usize = 0x2000;

pub struct Mbc5 {
    rom: Vec<u8>,
    ram: BackupFile,
    ram_enabled: bool,
    current_rom_bank: usize,
    current_ram_bank: usize,
    rom_banks: usize,
    ram_banks: usize,
}

impl Mbc5 {
    pub fn new(
        buffer: Vec<u8>,
        rom_banks: usize,
        ram_banks: usize,
        has_ram: bool,
        has_battery: bool,
        save_file: &Path,
    ) -> Result<Mbc5, CartridgeError> {
        let ram_banks = match has_ram {
            true => ram_banks,
            false => 0,
        };

        let ram = match has_battery {
            true => BackupFile::open(save_file, ram_banks * RAM_BANK_SIZE, 0x00)?,
            false => BackupFile::memory(ram_banks * RAM_BANK_SIZE, 0x00),
        };

        Ok(Mbc5 {
            rom: buffer,
            ram,
            ram_enabled: false,
            current_rom_bank: 1,
            current_ram_bank: 0,
            rom_banks,
            ram_banks,
        })
    }

    fn ram_offset(&self, address: u16) -> usize {
        self.current_ram_bank * RAM_BANK_SIZE | (address as usize & (RAM_BANK_SIZE - 1))
    }
}

impl SystemMemoryAccess for Mbc5 {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom_read(0, address),
            0x4000..=0x7FFF => self.rom_read(self.current_rom_bank, address),
            0xA000..=0xBFFF => {
                let offset = self.ram_offset(address);
                match self.ram_enabled && offset < self.ram.len() {
                    true => self.ram.read(offset),
                    false => 0xFF,
                }
            }
            _ => panic!("Invalid byte read for Mbc5: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
            0x2000..=0x2FFF => self.current_rom_bank = ((self.current_rom_bank & 0x100) | (value as usize)) % self.rom_banks,
            0x3000..=0x3FFF => {
                self.current_rom_bank = ((self.current_rom_bank & 0x0FF) | (((value & 0x1) as usize) << 8)) % self.rom_banks
            }
            0x4000..=0x5FFF => {
                if self.ram_banks > 0 {
                    self.current_ram_bank = ((value & 0x0F) as usize) % self.ram_banks;
                }
            }
            0x6000..=0x7FFF => {}
            0xA000..=0xBFFF => {
                let offset = self.ram_offset(address);
                if self.ram_enabled && offset < self.ram.len() {
                    self.ram.write(offset, value);
                }
            }
            _ => panic!("Invalid byte write for Mbc5: {:#06X}", address),
        }
    }
}

impl MemoryBankController for Mbc5 {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
