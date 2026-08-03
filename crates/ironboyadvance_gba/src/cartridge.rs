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
        flash::{Flash, FlashSize},
        gpio::Gpio,
        no_backup::NoBackup,
        rtc::Rtc,
        sram::Sram,
    },
    events::{CartridgeEvent, GbaEvent},
};

mod backup_file;
mod config;
mod eeprom;
mod flash;
mod gpio;
mod header;
mod no_backup;
mod rtc;
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

pub trait CartridgeBackup: SystemMemoryAccess<Address = u32> {
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
    gpio: Option<Gpio>,
}

impl Cartridge {
    pub fn load(
        rom_path: PathBuf,
        buffer: Vec<u8>,
        base_unix_seconds: u64,
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
                scheduler.clone(),
            )?),
            BackupType::Flash64KB => Box::new(Flash::new(buffer, &save_file, FlashSize::Small)?),
            BackupType::Flash128KB => Box::new(Flash::new(buffer, &save_file, FlashSize::Large)?),
        };

        let gpio = CartridgeDevice::Rtc
            .is_set(config.device_pattern())
            .then(|| Gpio::new(Some(Rtc::new(base_unix_seconds, rom_path.with_extension("rtc"), scheduler))));

        Ok(Cartridge { backup, gpio })
    }

    fn gpio_for_read(&self, address: u32) -> Option<&Gpio> {
        self.gpio.as_ref().filter(|gpio| gpio.readable() && Gpio::in_range(address))
    }

    fn gpio_for_write(&mut self, address: u32) -> Option<&mut Gpio> {
        self.gpio.as_mut().filter(|_| Gpio::in_range(address))
    }

    pub fn handle_event(&mut self, cartridge_event: CartridgeEvent) {
        self.backup.handle_event(cartridge_event);
    }
}

impl SystemMemoryAccess for Cartridge {
    type Address = u32;

    fn read_8(&self, address: u32) -> u8 {
        match self.gpio_for_read(address) {
            Some(gpio) => (gpio.read_16(address) >> ((address & 1) * 8)) as u8,
            None => self.backup.read_8(address),
        }
    }

    fn read_16(&self, address: u32) -> u16 {
        match self.gpio_for_read(address) {
            Some(gpio) => gpio.read_16(address),
            None => self.backup.read_16(address),
        }
    }

    fn read_32(&self, address: u32) -> u32 {
        match self.gpio_for_read(address) {
            Some(gpio) => gpio.read_16(address) as u32 | (gpio.read_16(address + 2) as u32) << 16,
            None => self.backup.read_32(address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match self.gpio_for_write(address) {
            Some(_) => {}
            None => self.backup.write_8(address, value),
        }
    }

    fn write_16(&mut self, address: u32, value: u16) {
        match self.gpio_for_write(address) {
            Some(gpio) => gpio.write_16(address, value),
            None => self.backup.write_16(address, value),
        }
    }

    fn write_32(&mut self, address: u32, value: u32) {
        match self.gpio_for_write(address) {
            Some(gpio) => {
                gpio.write_16(address, value as u16);
                gpio.write_16(address + 2, (value >> 16) as u16);
            }
            None => self.backup.write_32(address, value),
        }
    }
}
