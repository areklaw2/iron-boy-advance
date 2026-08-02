use std::path::PathBuf;

use getset::CopyGetters;
use ironboyadvance_common::memory::SystemMemoryAccess;
use ironboyadvance_sm83::GbMode;
use thiserror::Error;

use crate::cartridge::{
    header::{CartridgeType, Header},
    mbc1::Mbc1,
    mbc2::Mbc2,
    mbc3::Mbc3,
    mbc5::Mbc5,
    no_mbc::NoMbc,
};

mod backup_file;
mod header;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod no_mbc;
mod rtc;

const ROM_BANK_SIZE: usize = 0x4000;

#[derive(Error, Debug)]
pub enum CartridgeError {
    #[error("Unsupported Cartridge type")]
    InvalidCatridgeType,
    #[error("Existing save file has wrong size for detected cartridge")]
    SaveSizeMismatch,
    #[error("Save file I/O failed: {0}")]
    SaveIo(#[from] std::io::Error),
}

pub trait MemoryBankController: SystemMemoryAccess<Address = u16> {
    fn rom(&self) -> &[u8];

    fn rom_read(&self, bank: usize, address: u16) -> u8 {
        let offset = (bank * ROM_BANK_SIZE) | (address as usize & (ROM_BANK_SIZE - 1));
        *self.rom().get(offset).unwrap_or(&0xFF)
    }
}

#[derive(CopyGetters)]
pub struct Cartridge {
    mbc: Box<dyn MemoryBankController>,
    #[getset(get_copy = "pub")]
    mode: GbMode,
}

impl Cartridge {
    pub fn load(rom_file: PathBuf, buffer: Vec<u8>) -> Result<Cartridge, CartridgeError> {
        let header = Header::load(&buffer[0x000..=0x014F])?;
        let save_file = rom_file.with_extension("sav");
        let rom_banks = header.rom_banks();
        let ram_banks = header.ram_banks();

        let mbc: Box<dyn MemoryBankController> = match header.cartridge_type() {
            CartridgeType::NoMbc { ram, battery } => Box::new(NoMbc::new(buffer, ram, battery, &save_file)?),
            CartridgeType::Mbc1 { ram, battery } => {
                Box::new(Mbc1::new(buffer, rom_banks, ram_banks, ram, battery, &save_file)?)
            }
            CartridgeType::Mbc2 { battery } => Box::new(Mbc2::new(buffer, rom_banks, battery, &save_file)?),
            CartridgeType::Mbc3 { ram, battery, timer } => {
                Box::new(Mbc3::new(buffer, ram_banks, ram, battery, timer, &save_file)?)
            }
            CartridgeType::Mbc5 { ram, battery, .. } => {
                Box::new(Mbc5::new(buffer, rom_banks, ram_banks, ram, battery, &save_file)?)
            }
        };

        Ok(Cartridge {
            mbc,
            mode: header.mode(),
        })
    }
}

impl SystemMemoryAccess for Cartridge {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        self.mbc.read_8(address)
    }

    fn write_8(&mut self, address: u16, value: u8) {
        self.mbc.write_8(address, value)
    }
}
