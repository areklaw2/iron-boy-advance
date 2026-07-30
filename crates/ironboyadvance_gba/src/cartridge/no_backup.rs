use ironboyadvance_common::memory::SystemMemoryAccess;

use crate::cartridge::CartridgeBackup;

pub struct NoBackup {
    rom: Vec<u8>,
}

impl NoBackup {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl SystemMemoryAccess for NoBackup {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            0x08000000..=0x0DFFFFFF => self.rom_read(address),
            0x0E000000..=0x0FFFFFFF => 0,
            _ => panic!("Invalid byte read for NoBackup: {:08X}", address),
        }
    }

    fn write_8(&mut self, _address: u32, _value: u8) {}
}

impl CartridgeBackup for NoBackup {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
