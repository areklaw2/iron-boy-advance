use std::{cell::RefCell, path::PathBuf, rc::Rc};

use header::Header;
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use thiserror::Error;

use crate::{
    cartridge::{
        config::{
            BackupType::{self},
            CartridgeDevice, determine_cartridge_config,
        },
        eeprom::Eeprom,
        no_backup::NoBackup,
        sram::Sram,
    },
    events::{CartridgeEvent, GbaEvent},
};

mod backup_file;
mod config;
mod eeprom;
mod header;
mod no_backup;
mod sram;

#[derive(Error, Debug)]
pub enum CartridgeError {
    #[error("Unsupported Cartridge type")]
    InvalidCatridgeType,
    #[error("Existing save file has wrong size for detected backup")]
    SaveSizeMismatch,
    #[error("Save file I/O failed: {0}")]
    SaveIo(#[from] std::io::Error),
    #[error("Cartridge config has BackupType::Eeprom but no eeprom_size")]
    MissingEepromSize,
}

pub trait CartridgeBackup: SystemMemoryAccess {
    fn rom(&self) -> &[u8];

    fn rom_read(&self, address: u32) -> u8 {
        let offset = (address & 0x01FFFFFF) as usize;
        let rom = self.rom();
        match offset < rom.len() {
            true => rom[offset],
            false => (((address >> 1) & 0xFFFF) >> ((address & 1) * 8)) as u8,
        }
    }

    fn handle_event(&mut self, _cartridge_event: CartridgeEvent) {}
}

pub struct Cartridge {
    backup: Box<dyn CartridgeBackup>,
}

impl Cartridge {
    pub fn load(
        rom_path: PathBuf,
        buffer: Vec<u8>,
        scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
    ) -> Result<Cartridge, CartridgeError> {
        let header = Header::load(&buffer[0..228]);
        let config = determine_cartridge_config(&buffer, &header);
        let save_file = rom_path.with_extension("sav");

        let backup: Box<dyn CartridgeBackup> = match config.backup_type() {
            BackupType::None => Box::new(NoBackup::new(buffer)),
            BackupType::Sram => Box::new(Sram::new(buffer, &save_file)?),
            BackupType::Eeprom => Box::new(Eeprom::new(
                buffer,
                &save_file,
                config.eeprom_size().ok_or(CartridgeError::MissingEepromSize)?,
                scheduler,
            )?),
            BackupType::Flash64KB => todo!(),
            BackupType::Flash128KB => todo!(),
        };

        if CartridgeDevice::Rtc.is_set(config.device_pattern()) {
            println!("RTC")
        }

        Ok(Cartridge { backup })
    }

    pub fn handle_event(&mut self, cartridge_event: CartridgeEvent) {
        self.backup.handle_event(cartridge_event);
    }
}

impl SystemMemoryAccess for Cartridge {
    fn read_8(&self, address: u32) -> u8 {
        self.backup.read_8(address)
    }

    fn read_16(&self, address: u32) -> u16 {
        self.backup.read_16(address)
    }

    fn write_8(&mut self, address: u32, value: u8) {
        self.backup.write_8(address, value)
    }

    fn write_16(&mut self, address: u32, value: u16) {
        self.backup.write_16(address, value)
    }
}
