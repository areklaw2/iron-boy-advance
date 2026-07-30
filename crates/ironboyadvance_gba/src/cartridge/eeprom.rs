use std::cell::RefCell;
use std::path::Path;
use std::{cell::Cell, rc::Rc};

use ironboyadvance_common::bits::BitOps;
use ironboyadvance_common::memory::SystemMemoryAccess;
use ironboyadvance_common::scheduler::Scheduler;

use crate::cartridge::{CartridgeBackup, CartridgeError, backup_file::BackupFile};
use crate::events::{CartridgeEvent, GbaEvent};

const BLOCK_BYTES: u32 = 8;
const DUMMY_BITS: u8 = 4;
const STREAM_BITS: u8 = DUMMY_BITS + BLOCK_BYTES as u8 * 8;
const WRITE_CYCLES: usize = 108368;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EepromSize {
    Small,
    Large,
}

impl EepromSize {
    fn address_width(self) -> u8 {
        match self {
            EepromSize::Small => 6,
            EepromSize::Large => 14,
        }
    }

    fn backup_size(self) -> usize {
        match self {
            EepromSize::Small => 0x200,
            EepromSize::Large => 0x2000,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum AccessMode {
    Read,
    Write,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum TransferState {
    Idle,
    GetAccessMode,
    GetAddress {
        access_mode: AccessMode,
        shift: u32,
        bits_shifted: u8,
    },
    GetData {
        address: u32,
        shift: u64,
        bits_shifted: u8,
    },
    Stop {
        address: u32,
        data: Option<u64>,
    },
    Stream {
        data: u64,
        position: u8,
    },
    Busy,
}

pub struct Eeprom {
    rom: Vec<u8>,
    backup_file: BackupFile,
    size: EepromSize,
    state: Cell<TransferState>,
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
}

impl Eeprom {
    pub fn new(
        rom: Vec<u8>,
        save_file: &Path,
        size: EepromSize,
        scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
    ) -> Result<Self, CartridgeError> {
        Ok(Self {
            rom,
            backup_file: BackupFile::open(save_file, size.backup_size(), 0xFF)?,
            size,
            state: Cell::new(TransferState::Idle),
            scheduler,
        })
    }

    fn in_range(&self, address: u32) -> bool {
        if self.rom.len() > 0x1000000 {
            (0x0DFFFF00..=0x0DFFFFFF).contains(&address)
        } else {
            (0x0D000000..=0x0DFFFFFF).contains(&address)
        }
    }

    fn read_block(&self, address: u32) -> u64 {
        let offset = (address * BLOCK_BYTES) as usize;
        let mut value = 0u64;
        for i in 0..8 {
            value = (value << 8) | self.backup_file.read(offset + i) as u64;
        }
        value
    }

    fn write_block(&mut self, address: u32, data: u64) {
        let offset = (address * BLOCK_BYTES) as usize;
        for i in 0..8 {
            let shift = 56 - i * 8;
            self.backup_file.write(offset + i, (data >> shift) as u8);
        }
    }
}

impl SystemMemoryAccess for Eeprom {
    fn read_8(&self, address: u32) -> u8 {
        if self.in_range(address) {
            panic!("Only 16 bit reads accepted for Eeprom: {:08X}", address);
        }
        self.rom_read(address)
    }

    fn read_16(&self, address: u32) -> u16 {
        if !self.in_range(address) {
            return self.rom_read(address) as u16 | (self.rom_read(address + 1) as u16) << 8;
        }

        match self.state.get() {
            TransferState::Stream { data, position } => {
                let data_bit = if position < DUMMY_BITS {
                    0
                } else {
                    let bit = position - DUMMY_BITS;
                    data.bit((63 - bit) as usize) as u16
                };

                let next_position = position + 1;
                self.state.set(if next_position >= STREAM_BITS {
                    TransferState::Idle
                } else {
                    TransferState::Stream {
                        data,
                        position: next_position,
                    }
                });
                data_bit
            }
            TransferState::Busy => 0,
            _ => 1,
        }
    }

    fn write_8(&mut self, address: u32, _value: u8) {
        if self.in_range(address) {
            panic!("Only 16 bit writes accepted for Eeprom: {:08X}", address);
        }
    }

    fn write_16(&mut self, address: u32, value: u16) {
        if !self.in_range(address) {
            return;
        }

        let bit = (value & 1) as u8;

        match self.state.get() {
            TransferState::Idle => {
                let access_mode_started = bit == 1;
                if access_mode_started {
                    self.state.set(TransferState::GetAccessMode);
                }
            }
            TransferState::GetAccessMode => {
                let is_read_mode = bit == 1;
                let access_mode = if is_read_mode { AccessMode::Read } else { AccessMode::Write };
                self.state.set(TransferState::GetAddress {
                    access_mode,
                    shift: 0,
                    bits_shifted: 0,
                });
            }
            TransferState::GetAddress {
                access_mode,
                shift,
                bits_shifted,
            } => {
                let shift = (shift << 1) | bit as u32;
                let bits_shifted = bits_shifted + 1;
                if bits_shifted == self.size.address_width() {
                    self.state.set(match access_mode {
                        AccessMode::Read => TransferState::Stop {
                            address: shift,
                            data: None,
                        },
                        AccessMode::Write => TransferState::GetData {
                            address: shift,
                            shift: 0,
                            bits_shifted: 0,
                        },
                    });
                } else {
                    self.state.set(TransferState::GetAddress {
                        access_mode,
                        shift,
                        bits_shifted,
                    });
                }
            }
            TransferState::GetData {
                address,
                shift,
                bits_shifted,
            } => {
                let shift = (shift << 1) | bit as u64;
                let bits_shifted = bits_shifted + 1;
                self.state.set(if bits_shifted == 64 {
                    TransferState::Stop {
                        address,
                        data: Some(shift),
                    }
                } else {
                    TransferState::GetData {
                        address,
                        shift,
                        bits_shifted,
                    }
                });
            }
            TransferState::Stop { address, data } => match data {
                None => self.state.set(TransferState::Stream {
                    data: self.read_block(address),
                    position: 0,
                }),
                Some(data) => {
                    self.write_block(address, data);
                    self.state.set(TransferState::Busy);
                    self.scheduler
                        .borrow_mut()
                        .schedule((GbaEvent::Cartridge(CartridgeEvent::EepromReady), WRITE_CYCLES));
                }
            },
            TransferState::Stream { .. } | TransferState::Busy => {
                self.scheduler
                    .borrow_mut()
                    .cancel_events(GbaEvent::Cartridge(CartridgeEvent::EepromReady));
                let is_command_start = bit == 1;
                self.state.set(if is_command_start {
                    TransferState::GetAccessMode
                } else {
                    TransferState::Idle
                });
            }
        }
    }
}

impl CartridgeBackup for Eeprom {
    fn rom(&self) -> &[u8] {
        &self.rom
    }

    fn handle_event(&mut self, cartridge_event: CartridgeEvent) {
        if cartridge_event == CartridgeEvent::EepromReady && matches!(self.state.get(), TransferState::Busy) {
            self.state.set(TransferState::Idle)
        }
    }
}
