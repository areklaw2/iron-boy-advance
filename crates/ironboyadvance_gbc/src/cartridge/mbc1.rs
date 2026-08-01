use std::path::Path;

use ironboyadvance_common::memory::SystemMemoryAccess;

use super::{CartridgeError, MemoryBankController, backup_file::BackupFile};

const RAM_BANK_SIZE: usize = 0x2000;

pub struct Mbc1 {
    rom: Vec<u8>,
    ram: BackupFile,
    ram_enabled: bool,
    banking_mode: u8,
    current_rom_bank: usize,
    current_ram_bank: usize,
    rom_banks: usize,
    ram_banks: usize,
}

impl Mbc1 {
    pub fn new(
        buffer: Vec<u8>,
        rom_banks: usize,
        ram_banks: usize,
        has_ram: bool,
        has_battery: bool,
        save_file: &Path,
    ) -> Result<Mbc1, CartridgeError> {
        let ram_banks = match has_ram {
            true => ram_banks,
            false => 0,
        };

        let ram = match has_battery {
            true => BackupFile::open(save_file, ram_banks * RAM_BANK_SIZE, 0x00)?,
            false => BackupFile::memory(ram_banks * RAM_BANK_SIZE, 0x00),
        };

        Ok(Mbc1 {
            rom: buffer,
            ram,
            ram_enabled: false,
            banking_mode: 0,
            current_rom_bank: 1,
            current_ram_bank: 0,
            rom_banks,
            ram_banks,
        })
    }

    fn ram_offset(&self, address: u16) -> usize {
        let bank = match self.banking_mode {
            1 => self.current_ram_bank,
            _ => 0,
        };

        bank * RAM_BANK_SIZE | (address as usize & (RAM_BANK_SIZE - 1))
    }
}

impl SystemMemoryAccess for Mbc1 {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                let bank = match self.banking_mode {
                    0 => 0,
                    _ => self.current_rom_bank & 0xE0,
                };
                self.rom_read(bank, address)
            }
            0x4000..=0x7FFF => self.rom_read(self.current_rom_bank, address),
            0xA000..=0xBFFF => {
                let offset = self.ram_offset(address);
                match self.ram_enabled && offset < self.ram.len() {
                    true => self.ram.read(offset),
                    false => 0xFF,
                }
            }
            _ => panic!("Invalid byte read for Mbc1: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = value & 0xF == 0xA,
            0x2000..=0x3FFF => {
                let bank = match (value as usize) & 0x1F {
                    0 => 1,
                    n => n,
                };
                self.current_rom_bank = ((self.current_rom_bank & 0xE0) | bank) % self.rom_banks;
            }
            0x4000..=0x5FFF => {
                if self.rom_banks > 0x20 {
                    let bits = (value as usize & 0x03) % (self.rom_banks >> 5);
                    self.current_rom_bank = self.current_rom_bank & 0x1F | (bits << 5)
                }
                if self.ram_banks > 1 {
                    self.current_ram_bank = (value as usize) & 0x03;
                }
            }
            0x6000..=0x7FFF => self.banking_mode = value & 0x01,
            0xA000..=0xBFFF => {
                let offset = self.ram_offset(address);
                if self.ram_enabled && offset < self.ram.len() {
                    self.ram.write(offset, value);
                }
            }
            _ => panic!("Invalid byte write for Mbc1: {:#06X}", address),
        }
    }
}

impl MemoryBankController for Mbc1 {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
