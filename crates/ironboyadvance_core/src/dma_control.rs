use std::{cell::RefCell, rc::Rc};

use bitfields::bitfield;
use ironboyadvance_common::memory::SystemMemoryAccess;
use ironboyadvance_common::register_ops::RegisterOps;
use ironboyadvance_common::scheduler::Scheduler;

use crate::events::GbaEvent;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DestinationControl {
    Increment,
    Decrement,
    Fixed,
    Reload,
}

impl DestinationControl {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Increment,
            0x1 => Self::Decrement,
            0x2 => Self::Fixed,
            0x3 => Self::Reload,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SourceControl {
    Increment,
    Decrement,
    Fixed,
    Prohibited,
}

impl SourceControl {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Increment,
            0x1 => Self::Decrement,
            0x2 => Self::Fixed,
            0x3 => Self::Prohibited,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChunkSize {
    Size16,
    Size32,
}

impl ChunkSize {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Size16,
            0x1 => Self::Size32,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub fn bit_per_chunk(&self) -> usize {
        match self {
            ChunkSize::Size16 => 16,
            ChunkSize::Size32 => 32,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimingMode {
    Immediately,
    VBlank,
    HBlank,
    Special,
}

impl TimingMode {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Immediately,
            0x1 => Self::VBlank,
            0x2 => Self::HBlank,
            0x3 => Self::Special,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DmaControl {
    #[bits(5)]
    _not_used_0_4: u8,
    #[bits(2)]
    destination_control: DestinationControl,
    #[bits(2)]
    source_cotnrol: SourceControl,
    repeat: bool,
    #[bits(1)]
    chunk_size: ChunkSize,
    game_pak_drq: bool, //DMA3 only
    #[bits(2)]
    timing_mode: TimingMode,
    irq_enabled: bool,
    enabled: bool,
}

impl RegisterOps<u16> for DmaControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[derive(Copy, Clone)]
pub struct DmaChannel {
    source_address: u32,
    current_source_address: u32,
    destination_address: u32,
    current_destination_address: u32,
    count: u16,
    current_count: u16,
    control: DmaControl,
}

impl DmaChannel {
    pub fn new() -> Self {
        Self {
            source_address: 0,
            current_source_address: 0,
            destination_address: 0,
            current_destination_address: 0,
            count: 0,
            current_count: 0,
            control: DmaControl::from_bits(0),
        }
    }

    pub fn write_control(&mut self, address: u32, value: u8) {}
}

pub struct DmaController {
    channels: [DmaChannel; 4],
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
}

impl DmaController {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbaEvent>>>) -> Self {
        Self {
            channels: [DmaChannel::new(); 4],
            scheduler,
        }
    }

    pub fn is_active(&self) -> bool {
        false
    }
}

impl SystemMemoryAccess for DmaController {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // DMA0SAD, DMA0DAD, DMA0CNT_L, DMA0CNT_H
            0x040000B0..=0x040000B3 | 0x040000B4..=0x040000B7 | 0x040000B8..=0x040000B9 => 0,
            0x040000BA..=0x040000BB => self.channels[0].control.read_byte(address),
            // DMA1SAD, DMA1DAD, DMA1CNT_L, DMA1CNT_H
            0x040000BC..=0x040000BF | 0x040000C0..=0x040000C3 | 0x040000C4..=0x040000C5 => 0,
            0x040000C6..=0x040000C7 => self.channels[1].control.read_byte(address),
            // DMA2SAD, DMA2DAD, DMA2CNT_L, DMA2CNT_H
            0x040000C8..=0x040000CB | 0x040000CC..=0x040000CF | 0x040000D0..=0x040000D1 => 0,
            0x040000D2..=0x040000D3 => self.channels[2].control.read_byte(address),
            // DMA3SAD, DMA3DAD, DMA3CNT_L, DMA3CNT_H
            0x040000D4..=0x040000D7 | 0x040000D8..=0x040000DB | 0x040000DC..=0x040000DD => 0,
            0x040000DE..=0x040000DF => self.channels[3].control.read_byte(address),
            _ => panic!("Invalid byte read for DmaController: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // DMA0SAD, DMA0DAD, DMA0CNT_L, DMA0CNT_H
            0x040000B0..=0x040000B3 => self.channels[0].source_address.write_byte(address, value),
            0x040000B4..=0x040000B7 => self.channels[0].destination_address.write_byte(address, value),
            0x040000B8..=0x040000B9 => self.channels[0].count.write_byte(address, value),
            0x040000BA..=0x040000BB => self.channels[0].write_control(address, value),
            // DMA1SAD, DMA1DAD, DMA1CNT_L, DMA1CNT_H
            0x040000BC..=0x040000BF => self.channels[1].source_address.write_byte(address, value),
            0x040000C0..=0x040000C3 => self.channels[1].destination_address.write_byte(address, value),
            0x040000C4..=0x040000C5 => self.channels[1].count.write_byte(address, value),
            0x040000C6..=0x040000C7 => self.channels[1].write_control(address, value),
            // DMA2SAD, DMA2DAD, DMA2CNT_L, DMA2CNT_H
            0x040000C8..=0x040000CB => self.channels[2].source_address.write_byte(address, value),
            0x040000CC..=0x040000CF => self.channels[2].destination_address.write_byte(address, value),
            0x040000D0..=0x040000D1 => self.channels[2].count.write_byte(address, value),
            0x040000D2..=0x040000D3 => self.channels[2].write_control(address, value),
            // DMA3SAD, DMA3DAD, DMA3CNT_L, DMA3CNT_H
            0x040000D4..=0x040000D7 => self.channels[3].source_address.write_byte(address, value),
            0x040000D8..=0x040000DB => self.channels[3].destination_address.write_byte(address, value),
            0x040000DC..=0x040000DD => self.channels[3].count.write_byte(address, value),
            0x040000DE..=0x040000DF => self.channels[3].write_control(address, value),
            _ => panic!("Invalid byte read for DmaController: {:#010X}", address),
        }
    }
}
