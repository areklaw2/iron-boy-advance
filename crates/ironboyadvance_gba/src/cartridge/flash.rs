use std::path::Path;

use ironboyadvance_common::memory::SystemMemoryAccess;

use crate::cartridge::{CartridgeBackup, CartridgeError, backup_file::BackupFile};

const BANK_SIZE: usize = 64 * 1024;
const SECTOR_SIZE: usize = 4 * 1024;

const COMMAND_ADDRESS_1: u32 = 0x0E005555;
const COMMAND_ADDRESS_2: u32 = 0x0E002AAA;

const UNLOCK_FIRST: u8 = 0xAA;
const UNLOCK_SECOND: u8 = 0x55;
const ENTER_ID_MODE: u8 = 0x90;
const EXIT_ID_MODE: u8 = 0xF0;
const PREPARE_ERASE: u8 = 0x80;
const ERASE_CHIP: u8 = 0x10;
const ERASE_SECTOR: u8 = 0x30;
const PREPARE_WRITE_BYTE: u8 = 0xA0;
const SELECT_BANK: u8 = 0xB0;

const PANASONIC_DEVICE_ID: [u8; 2] = [0x32, 0x1B];
const SANYO_DEVICE_ID: [u8; 2] = [0x62, 0x13];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FlashSize {
    Small,
    Large,
}

impl FlashSize {
    fn backup_size(self) -> usize {
        match self {
            FlashSize::Small => BANK_SIZE,
            FlashSize::Large => BANK_SIZE * 2,
        }
    }

    fn device_id(self) -> [u8; 2] {
        match self {
            FlashSize::Small => PANASONIC_DEVICE_ID,
            FlashSize::Large => SANYO_DEVICE_ID,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum FlashState {
    Ready,
    ReceivedFirstUnlock,
    AwaitingCommand,
    ErasePrepared,
    EraseReceivedFirstUnlock,
    AwaitingEraseCommand,
    AwaitingProgramValue,
    AwaitingBankNumber,
}

pub struct Flash {
    rom: Vec<u8>,
    backup_file: BackupFile,
    size: FlashSize,
    state: FlashState,
    bank: usize,
    in_id_mode: bool,
}

impl Flash {
    pub fn new(rom: Vec<u8>, save_file: &Path, size: FlashSize) -> Result<Self, CartridgeError> {
        Ok(Self {
            rom,
            backup_file: BackupFile::open(save_file, size.backup_size(), 0xFF)?,
            size,
            state: FlashState::Ready,
            bank: 0,
            in_id_mode: false,
        })
    }

    fn in_range(&self, address: u32) -> bool {
        (0x0E000000..=0x0FFFFFFF).contains(&address)
    }

    fn backup_offset(&self, address: u32) -> usize {
        self.bank * BANK_SIZE + (address & 0xFFFF) as usize
    }

    fn handle_write(&mut self, address: u32, value: u8) {
        self.state = match self.state {
            FlashState::Ready => match (address, value) {
                (COMMAND_ADDRESS_1, UNLOCK_FIRST) => FlashState::ReceivedFirstUnlock,
                _ => FlashState::Ready,
            },
            FlashState::ReceivedFirstUnlock => match (address, value) {
                (COMMAND_ADDRESS_2, UNLOCK_SECOND) => FlashState::AwaitingCommand,
                _ => FlashState::Ready,
            },
            FlashState::AwaitingCommand => self.execute_command(address, value),
            FlashState::ErasePrepared => match (address, value) {
                (COMMAND_ADDRESS_1, UNLOCK_FIRST) => FlashState::EraseReceivedFirstUnlock,
                _ => FlashState::Ready,
            },
            FlashState::EraseReceivedFirstUnlock => match (address, value) {
                (COMMAND_ADDRESS_2, UNLOCK_SECOND) => FlashState::AwaitingEraseCommand,
                _ => FlashState::Ready,
            },
            FlashState::AwaitingEraseCommand => self.execute_erase_command(address, value),
            FlashState::AwaitingProgramValue => {
                let backup_offset = self.backup_offset(address);
                self.backup_file.write(backup_offset, value);
                FlashState::Ready
            }
            FlashState::AwaitingBankNumber => {
                self.bank = (value & 1) as usize;
                FlashState::Ready
            }
        };
    }

    fn execute_command(&mut self, address: u32, value: u8) -> FlashState {
        if address != COMMAND_ADDRESS_1 {
            return FlashState::Ready;
        }

        match value {
            ENTER_ID_MODE => {
                self.in_id_mode = true;
                FlashState::Ready
            }
            EXIT_ID_MODE => {
                self.in_id_mode = false;
                FlashState::Ready
            }
            PREPARE_ERASE => FlashState::ErasePrepared,
            PREPARE_WRITE_BYTE => FlashState::AwaitingProgramValue,
            SELECT_BANK if self.size == FlashSize::Large => FlashState::AwaitingBankNumber,
            _ => FlashState::Ready,
        }
    }

    fn execute_erase_command(&mut self, address: u32, value: u8) -> FlashState {
        match (address, value) {
            (COMMAND_ADDRESS_1, ERASE_CHIP) => self.backup_file.fill(0, self.size.backup_size(), 0xFF),
            (_, ERASE_SECTOR) => {
                let backup_offset = self.backup_offset(address) & !(SECTOR_SIZE - 1);
                self.backup_file.fill(backup_offset, SECTOR_SIZE, 0xFF);
            }
            _ => {}
        }

        FlashState::Ready
    }
}

impl SystemMemoryAccess for Flash {
    type Address = u32;

    fn read_8(&self, address: u32) -> u8 {
        match address {
            0x08000000..=0x0DFFFFFF => self.rom_read(address),
            0x0E000000..=0x0FFFFFFF => match self.in_id_mode && address & 0xFFFF < 2 {
                true => self.size.device_id()[(address & 1) as usize],
                false => self.backup_file.read(self.backup_offset(address)),
            },
            _ => panic!("Invalid byte read for Flash: {:08X}", address),
        }
    }

    fn read_16(&self, address: u32) -> u16 {
        match self.in_range(address) {
            true => u16::from_le_bytes([self.read_8(address); 2]),
            false => self.rom_read(address) as u16 | (self.rom_read(address + 1) as u16) << 8,
        }
    }

    fn read_32(&self, address: u32) -> u32 {
        match self.in_range(address) {
            true => u32::from_le_bytes([self.read_8(address); 4]),
            false => self.read_16(address) as u32 | (self.read_16(address + 2) as u32) << 16,
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            0x08000000..=0x0DFFFFFF => {}
            0x0E000000..=0x0FFFFFFF => self.handle_write(0x0E000000 | (address & 0xFFFF), value),
            _ => panic!("Invalid byte write for Flash: {:08X}", address),
        }
    }

    fn write_16(&mut self, address: u32, value: u16) {
        if self.in_range(address) {
            self.write_8(address, (value >> ((address & 1) * 8)) as u8);
        }
    }

    fn write_32(&mut self, address: u32, value: u32) {
        if self.in_range(address) {
            self.write_8(address, (value >> ((address & 3) * 8)) as u8);
        }
    }
}

impl CartridgeBackup for Flash {
    fn rom(&self) -> &[u8] {
        &self.rom
    }
}
