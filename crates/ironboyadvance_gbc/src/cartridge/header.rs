use getset::CopyGetters;
use tracing::warn;

use super::CartridgeError;
use crate::GbMode;

const CHECKSUM_START: usize = 0x0134;
const CHECKSUM_END: usize = 0x014C;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartridgeType {
    NoMbc { ram: bool, battery: bool },
    Mbc1 { ram: bool, battery: bool },
    Mbc2 { battery: bool },
    Mbc3 { ram: bool, battery: bool, timer: bool },
    Mbc5 { ram: bool, battery: bool, rumble: bool },
}

impl TryFrom<u8> for CartridgeType {
    type Error = CartridgeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(CartridgeType::NoMbc {
                ram: false,
                battery: false,
            }),
            0x08 => Ok(CartridgeType::NoMbc {
                ram: true,
                battery: false,
            }),
            0x09 => Ok(CartridgeType::NoMbc {
                ram: true,
                battery: true,
            }),
            0x01 => Ok(CartridgeType::Mbc1 {
                ram: false,
                battery: false,
            }),
            0x02 => Ok(CartridgeType::Mbc1 {
                ram: true,
                battery: false,
            }),
            0x03 => Ok(CartridgeType::Mbc1 {
                ram: true,
                battery: true,
            }),
            0x05 => Ok(CartridgeType::Mbc2 { battery: false }),
            0x06 => Ok(CartridgeType::Mbc2 { battery: true }),
            0x0F => Ok(CartridgeType::Mbc3 {
                ram: false,
                battery: true,
                timer: true,
            }),
            0x10 => Ok(CartridgeType::Mbc3 {
                ram: true,
                battery: true,
                timer: true,
            }),
            0x11 => Ok(CartridgeType::Mbc3 {
                ram: false,
                battery: false,
                timer: false,
            }),
            0x12 => Ok(CartridgeType::Mbc3 {
                ram: true,
                battery: false,
                timer: false,
            }),
            0x13 => Ok(CartridgeType::Mbc3 {
                ram: true,
                battery: true,
                timer: false,
            }),
            0x19 => Ok(CartridgeType::Mbc5 {
                ram: false,
                battery: false,
                rumble: false,
            }),
            0x1A => Ok(CartridgeType::Mbc5 {
                ram: true,
                battery: false,
                rumble: false,
            }),
            0x1B => Ok(CartridgeType::Mbc5 {
                ram: true,
                battery: true,
                rumble: false,
            }),
            0x1C => Ok(CartridgeType::Mbc5 {
                ram: false,
                battery: false,
                rumble: true,
            }),
            0x1D => Ok(CartridgeType::Mbc5 {
                ram: true,
                battery: false,
                rumble: true,
            }),
            0x1E => Ok(CartridgeType::Mbc5 {
                ram: true,
                battery: true,
                rumble: true,
            }),
            _ => Err(CartridgeError::InvalidCatridgeType),
        }
    }
}

#[derive(CopyGetters)]
pub struct Header {
    cgb_flag: u8,
    #[getset(get_copy = "pub")]
    cartridge_type: CartridgeType,
    rom_size: u8,
    ram_size: u8,
    #[getset(get_copy = "pub")]
    checksum: u8,
}

impl Header {
    pub fn load(bytes: &[u8]) -> Result<Header, CartridgeError> {
        let header = Header {
            cgb_flag: bytes[0x0143],
            cartridge_type: CartridgeType::try_from(bytes[0x0147])?,
            rom_size: bytes[0x0148],
            ram_size: bytes[0x0149],
            checksum: bytes[0x014D],
        };

        let mut checksum: u8 = 0;
        for address in CHECKSUM_START..=CHECKSUM_END {
            checksum = checksum.wrapping_sub(bytes[address]).wrapping_sub(1)
        }

        if checksum != header.checksum {
            warn!(
                "Header checksum mismatch: expected {:#04X}, calculated {:#04X}",
                header.checksum, checksum
            );
        }

        Ok(header)
    }

    pub fn mode(&self) -> GbMode {
        match self.cgb_flag & 0x80 != 0 {
            true => GbMode::Color,
            false => GbMode::ColorAsMonochrome,
        }
    }

    pub fn rom_banks(&self) -> usize {
        if self.rom_size <= 8 { 2 << self.rom_size } else { 0 }
    }

    pub fn ram_banks(&self) -> usize {
        match self.ram_size {
            0x1 | 0x2 => 1,
            0x3 => 4,
            0x4 => 16,
            0x5 => 8,
            _ => 0,
        }
    }
}
