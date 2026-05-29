use std::{cell::RefCell, rc::Rc};

use bitfields::bitfield;
use ironboyadvance_common::memory::{MemoryAccess, SystemMemoryAccess};
use ironboyadvance_common::register_ops::RegisterOps;
use ironboyadvance_common::scheduler::Scheduler;

use crate::events::{GbaEvent, InterruptEvent};
use crate::system_bus::ROM_WS0_LO;

const DMA3_MAX_TRANSFER_COUNT: u32 = 0x10000;
const DMA_MAX_TRANSFER_COUNT: u32 = 0x4000;

const DMA_OVERFLOW_INTERRUPTS: [InterruptEvent; 4] = [
    InterruptEvent::Dma0Overflow,
    InterruptEvent::Dma1Overflow,
    InterruptEvent::Dma2Overflow,
    InterruptEvent::Dma3Overflow,
];

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

    pub fn step(&self, bytes: i32) -> i32 {
        match self {
            DestinationControl::Increment | DestinationControl::Reload => bytes,
            DestinationControl::Decrement => -bytes,
            DestinationControl::Fixed => 0,
        }
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

    pub fn step(&self, bytes: i32) -> i32 {
        match self {
            SourceControl::Increment => bytes,
            SourceControl::Decrement => -bytes,
            SourceControl::Fixed | SourceControl::Prohibited => 0,
        }
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

    pub fn bytes(&self) -> i32 {
        match self {
            ChunkSize::Size16 => 2,
            ChunkSize::Size32 => 4,
        }
    }

    pub fn address_alignment(&self) -> u32 {
        match self {
            ChunkSize::Size16 => !1,
            ChunkSize::Size32 => !3,
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
    source_control: SourceControl,
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
pub struct DmaTransfer {
    pub source: u32,
    pub destination: u32,
    pub source_access: u8,
    pub destination_access: u8,
    pub chunk_size: ChunkSize,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RequestType {
    HBlank,
    VBlank,
    FifoA,
    FifoB,
    Video,
}

#[derive(Copy, Clone)]
pub struct DmaChannel {
    id: usize,
    source_address: u32,
    current_source_address: u32,
    destination_address: u32,
    current_destination_address: u32,
    count: u16,
    current_count: u32,
    control: DmaControl,
    is_fifo: bool,
    accessed_rom: bool,
}

impl DmaChannel {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            source_address: 0,
            current_source_address: 0,
            destination_address: 0,
            current_destination_address: 0,
            count: 0,
            current_count: 0,
            control: DmaControl::from_bits(0),
            is_fifo: false,
            accessed_rom: false,
        }
    }

    fn write_source_address(&mut self, address: u32, value: u8) {
        self.source_address.write_byte(address, value);
        self.source_address &= match self.id {
            0 => 0x07FF_FFFF,
            _ => 0x0FFF_FFFF,
        };
    }

    fn write_destination_address(&mut self, address: u32, value: u8) {
        self.destination_address.write_byte(address, value);
        self.destination_address &= match self.id {
            3 => 0x0FFF_FFFF,
            _ => 0x07FF_FFFF,
        };
    }

    fn write_count(&mut self, address: u32, value: u8) {
        self.count.write_byte(address, value);
        self.count &= match self.id {
            3 => 0xFFFF,
            _ => 0x3FFF,
        };
    }

    fn write_control(&mut self, address: u32, value: u8) -> bool {
        let was_enabled = self.control.enabled();
        self.control.write_byte(address, value);

        let enabled = !was_enabled && self.control.enabled();
        if enabled {
            self.is_fifo = self.control.timing_mode() == TimingMode::Special && (self.id == 1 || self.id == 2);

            let alignment = match self.is_fifo {
                true => !3,
                false => self.control.chunk_size().address_alignment(),
            };
            self.current_source_address = self.source_address & alignment;
            self.current_destination_address = self.destination_address & alignment;

            self.update_count();
            self.accessed_rom = false;
        }
        enabled
    }

    fn reload(&mut self) {
        if !self.is_fifo && self.control.destination_control() == DestinationControl::Reload {
            self.current_destination_address = self.destination_address & self.control.chunk_size().address_alignment();
        }
        self.update_count();
        self.accessed_rom = false;
    }

    fn effective_chunk_size(&self) -> ChunkSize {
        match self.is_fifo {
            true => ChunkSize::Size32,
            false => self.control.chunk_size(),
        }
    }

    fn update_count(&mut self) {
        self.current_count = match self.is_fifo {
            true => 4,
            false => {
                let max_count = match self.id == 3 {
                    true => DMA3_MAX_TRANSFER_COUNT,
                    false => DMA_MAX_TRANSFER_COUNT,
                };
                match self.count as u32 {
                    0 => max_count,
                    count => count,
                }
            }
        };
    }
}

pub struct DmaController {
    channels: [DmaChannel; 4],
    runnable: [bool; 4],
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
}

impl DmaController {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbaEvent>>>) -> Self {
        Self {
            channels: std::array::from_fn(DmaChannel::new),
            runnable: [false; 4],
            scheduler,
        }
    }

    fn active_channel(&self) -> Option<usize> {
        self.runnable.iter().position(|&runnable| runnable)
    }

    pub fn is_active(&self) -> bool {
        self.runnable.iter().any(|&runnable| runnable)
    }

    fn write_control(&mut self, channel_id: usize, address: u32, value: u8) {
        if self.channels[channel_id].write_control(address, value)
            && self.channels[channel_id].control.timing_mode() == TimingMode::Immediately
        {
            self.scheduler.borrow_mut().schedule((GbaEvent::Dma(channel_id), 2));
        }
    }

    pub fn request_dma(&mut self, request_type: RequestType) {
        let (candidates, expected_timing): (&[usize], TimingMode) = match request_type {
            RequestType::HBlank => (&[0, 1, 2, 3], TimingMode::HBlank),
            RequestType::VBlank => (&[0, 1, 2, 3], TimingMode::VBlank),
            RequestType::FifoA => (&[1], TimingMode::Special),
            RequestType::FifoB => (&[2], TimingMode::Special),
            RequestType::Video => (&[3], TimingMode::Special),
        };

        for &id in candidates {
            let control = self.channels[id].control;
            if control.enabled() && control.timing_mode() == expected_timing {
                self.scheduler.borrow_mut().schedule((GbaEvent::Dma(id), 2));
            }
        }
    }

    pub fn stop_video_transfer(&mut self) {
        let control = self.channels[3].control;
        if control.enabled() && control.timing_mode() == TimingMode::Special {
            self.channels[3].control.set_enabled(false);
        }
    }

    pub fn pending_dma_request(&self) -> Option<usize> {
        let mut scheduler = self.scheduler.borrow_mut();
        match scheduler.peek() {
            Some(GbaEvent::Dma(_)) => {
                let (GbaEvent::Dma(id), _) = scheduler.pop()? else {
                    return None;
                };
                Some(id)
            }
            _ => None,
        }
    }

    pub fn handle_event(&mut self, id: usize) {
        let previous_active = self.active_channel();
        self.runnable[id] = true;
        if let Some(previous_id) = previous_active
            && id < previous_id
        {
            self.channels[previous_id].accessed_rom = false;
        }
    }

    pub fn next_transfer(&self) -> Option<DmaTransfer> {
        let channel = &self.channels[self.active_channel()?];
        let chunk_size = channel.effective_chunk_size();

        let mut source_access = MemoryAccess::Sequential | MemoryAccess::Dma;
        let mut destination_access = MemoryAccess::Sequential | MemoryAccess::Dma;
        if !channel.accessed_rom {
            if channel.current_source_address >= ROM_WS0_LO {
                source_access = MemoryAccess::NonSequential | MemoryAccess::Dma;
            } else if channel.current_destination_address >= ROM_WS0_LO {
                destination_access = MemoryAccess::NonSequential | MemoryAccess::Dma;
            }
        }

        Some(DmaTransfer {
            source: channel.current_source_address,
            destination: channel.current_destination_address,
            source_access,
            destination_access,
            chunk_size,
        })
    }

    pub fn complete_transfer(&mut self) {
        let Some(id) = self.active_channel() else { return };
        let channel = &mut self.channels[id];

        let bytes = channel.effective_chunk_size().bytes();
        let source_step = channel.control.source_control().step(bytes);
        let destination_step = match channel.is_fifo {
            true => 0,
            false => channel.control.destination_control().step(bytes),
        };

        if !channel.accessed_rom
            && (channel.current_source_address >= ROM_WS0_LO || channel.current_destination_address >= ROM_WS0_LO)
        {
            channel.accessed_rom = true;
        }

        channel.current_source_address = channel.current_source_address.wrapping_add(source_step as u32);
        channel.current_destination_address = channel.current_destination_address.wrapping_add(destination_step as u32);
        channel.current_count -= 1;

        if channel.current_count == 0 {
            self.runnable[id] = false;
            let channel = &mut self.channels[id];
            let irq_enabled = channel.control.irq_enabled();
            let repeat = channel.control.repeat() && channel.control.timing_mode() != TimingMode::Immediately;
            match repeat {
                true => channel.reload(),
                false => channel.control.set_enabled(false),
            }

            if irq_enabled {
                self.scheduler
                    .borrow_mut()
                    .schedule((GbaEvent::Interrupt(DMA_OVERFLOW_INTERRUPTS[id]), 0));
            }
        }
    }
}

impl SystemMemoryAccess for DmaController {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // DMA0SAD, DMA0DAD, DMA0CNT_L, DMA0CNT_H
            0x040000B0..=0x040000B9 => 0,
            0x040000BA..=0x040000BB => self.channels[0].control.read_byte(address),
            // DMA1SAD, DMA1DAD, DMA1CNT_L, DMA1CNT_H
            0x040000BC..=0x040000C5 => 0,
            0x040000C6..=0x040000C7 => self.channels[1].control.read_byte(address),
            // DMA2SAD, DMA2DAD, DMA2CNT_L, DMA2CNT_H
            0x040000C8..=0x040000D1 => 0,
            0x040000D2..=0x040000D3 => self.channels[2].control.read_byte(address),
            // DMA3SAD, DMA3DAD, DMA3CNT_L, DMA3CNT_H
            0x040000D4..=0x040000DD => 0,
            0x040000DE..=0x040000DF => self.channels[3].control.read_byte(address),
            _ => panic!("Invalid byte read for DmaController: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // DMA0SAD, DMA0DAD, DMA0CNT_L, DMA0CNT_H
            0x040000B0..=0x040000B3 => self.channels[0].write_source_address(address, value),
            0x040000B4..=0x040000B7 => self.channels[0].write_destination_address(address, value),
            0x040000B8..=0x040000B9 => self.channels[0].write_count(address, value),
            0x040000BA..=0x040000BB => self.write_control(0, address, value),
            // DMA1SAD, DMA1DAD, DMA1CNT_L, DMA1CNT_H
            0x040000BC..=0x040000BF => self.channels[1].write_source_address(address, value),
            0x040000C0..=0x040000C3 => self.channels[1].write_destination_address(address, value),
            0x040000C4..=0x040000C5 => self.channels[1].write_count(address, value),
            0x040000C6..=0x040000C7 => self.write_control(1, address, value),
            // DMA2SAD, DMA2DAD, DMA2CNT_L, DMA2CNT_H
            0x040000C8..=0x040000CB => self.channels[2].write_source_address(address, value),
            0x040000CC..=0x040000CF => self.channels[2].write_destination_address(address, value),
            0x040000D0..=0x040000D1 => self.channels[2].write_count(address, value),
            0x040000D2..=0x040000D3 => self.write_control(2, address, value),
            // DMA3SAD, DMA3DAD, DMA3CNT_L, DMA3CNT_H
            0x040000D4..=0x040000D7 => self.channels[3].write_source_address(address, value),
            0x040000D8..=0x040000DB => self.channels[3].write_destination_address(address, value),
            0x040000DC..=0x040000DD => self.channels[3].write_count(address, value),
            0x040000DE..=0x040000DF => self.write_control(3, address, value),
            _ => panic!("Invalid byte write for DmaController: {:#010X}", address),
        }
    }
}
