use std::path::Path;

use ironboyadvance_common::memory::SystemMemoryAccess;

use super::rtc::RealTimeClock;
use super::{CartridgeError, MemoryBankController, backup_file::BackupFile};

const RAM_BANK_SIZE: usize = 0x2000;
const CLOCK_BYTES: usize = 8;
const CLOCK_REGISTERS: usize = 5;

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: BackupFile,
    ram_enabled: bool,
    current_rom_bank: usize,
    current_ram_bank: usize,
    ram_banks: usize,
    select_rtc_register: bool,
    rtc: RealTimeClock,
}

impl Mbc3 {
    pub fn new(
        buffer: Vec<u8>,
        ram_banks: usize,
        has_ram: bool,
        has_battery: bool,
        has_real_time_clock: bool,
        save_file: &Path,
    ) -> Result<Mbc3, CartridgeError> {
        let ram_banks = match has_ram {
            true => ram_banks,
            false => 0,
        };

        let clock_bytes = match has_real_time_clock {
            true => CLOCK_BYTES,
            false => 0,
        };

        let size = ram_banks * RAM_BANK_SIZE + clock_bytes;
        let ram = match has_battery {
            true => BackupFile::open(save_file, size, 0x00)?,
            false => BackupFile::memory(size, 0x00),
        };

        let mut rtc = RealTimeClock::new(has_real_time_clock);
        if has_real_time_clock && rtc.time().is_some() {
            let mut clock = [0; CLOCK_BYTES];
            for (offset, byte) in clock.iter_mut().enumerate() {
                *byte = ram.read(ram_banks * RAM_BANK_SIZE + offset);
            }
            rtc.load_time(Some(u64::from_be_bytes(clock)));
        }

        Ok(Mbc3 {
            rom: buffer,
            ram,
            ram_enabled: false,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_banks,
            select_rtc_register: false,
            rtc,
        })
    }

    fn ram_offset(&self, address: u16) -> usize {
        (self.current_ram_bank * RAM_BANK_SIZE) | (address as usize & (RAM_BANK_SIZE - 1))
    }

    fn persist_clock(&mut self) {
        let Some(time) = self.rtc.time() else {
            return;
        };

        let base = self.ram_banks * RAM_BANK_SIZE;
        for (offset, byte) in time.to_be_bytes().iter().enumerate() {
            self.ram.write(base + offset, *byte);
        }
    }
}

impl SystemMemoryAccess for Mbc3 {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom_read(0, address),
            0x4000..=0x7FFF => self.rom_read(self.current_rom_bank, address),
            0xA000..=0xBFFF => match (self.ram_enabled, self.select_rtc_register) {
                (false, _) => 0xFF,
                (true, false) if self.current_ram_bank < self.ram_banks => self.ram.read(self.ram_offset(address)),
                (true, true) if self.current_ram_bank < CLOCK_REGISTERS => self.rtc.latch_register(self.current_ram_bank),
                _ => 0xFF,
            },
            _ => panic!("Invalid byte read for Mbc3: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
            0x2000..=0x3FFF => {
                self.current_rom_bank = match value & 0x7F {
                    0 => 1,
                    n => n as usize,
                }
            }
            0x4000..=0x5FFF => {
                self.select_rtc_register = value & 0x8 == 0x8;
                self.current_ram_bank = (value & 0x7) as usize;
            }
            0x6000..=0x7FFF => self.rtc.set_latch_registers(),
            0xA000..=0xBFFF => match (self.ram_enabled, self.select_rtc_register) {
                (false, _) => {}
                (true, false) if self.current_ram_bank < self.ram_banks => {
                    let offset = self.ram_offset(address);
                    self.ram.write(offset, value);
                }
                (true, true) if self.current_ram_bank < CLOCK_REGISTERS => {
                    self.rtc.set_registers();
                    let register_mask = match self.current_ram_bank {
                        0 | 1 => 0x3F,
                        2 => 0x1F,
                        4 => 0xC1,
                        _ => 0xFF,
                    };
                    self.rtc.set_register(self.current_ram_bank, value & register_mask);
                    self.rtc.set_time();
                    self.persist_clock();
                }
                _ => {}
            },
            _ => panic!("Invalid byte write for Mbc3: {:#06X}", address),
        }
    }
}

impl MemoryBankController for Mbc3 {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
