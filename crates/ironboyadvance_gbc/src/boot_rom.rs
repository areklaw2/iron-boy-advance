use getset::CopyGetters;
use ironboyadvance_common::memory::SystemMemoryAccess;
use thiserror::Error;

const DMG_BOOT_ROM_SIZE: usize = 0x0100;
const CGB_BOOT_ROM_SIZE: usize = 0x0900;
const CARTRIDGE_HEADER_START: u16 = 0x0100;
const CARTRIDGE_HEADER_END: u16 = 0x01FF;
const BOOT_ROM_DISABLED: u8 = 0xFE;

#[derive(Error, Debug)]
pub enum BootRomError {
    #[error("Invalid boot rom length: {0}")]
    InvalidBootRomLength(usize),
}

#[derive(Debug, CopyGetters)]
pub struct BootRom {
    data: Vec<u8>,
    #[getset(get_copy = "pub")]
    loaded: bool,
    #[getset(get_copy = "pub")]
    mapped: bool,
}

impl BootRom {
    pub fn load(buffer: Vec<u8>) -> Result<BootRom, BootRomError> {
        let (data, loaded) = match buffer.len() {
            0 => (Vec::new(), false),
            DMG_BOOT_ROM_SIZE | CGB_BOOT_ROM_SIZE => (buffer, true),
            length => return Err(BootRomError::InvalidBootRomLength(length)),
        };

        Ok(BootRom {
            data,
            loaded,
            mapped: loaded,
        })
    }

    pub fn contains(&self, address: u16) -> bool {
        self.mapped
            && (address as usize) < self.data.len()
            && !(CARTRIDGE_HEADER_START..=CARTRIDGE_HEADER_END).contains(&address)
    }
}

impl SystemMemoryAccess for BootRom {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF50 => match self.mapped {
                true => BOOT_ROM_DISABLED,
                false => 0xFF,
            },
            _ => self.data[address as usize],
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF50 => self.mapped &= value & 0x01 == 0,
            _ => panic!("Invalid byte write for BootRom: {:#06X}", address),
        }
    }
}
