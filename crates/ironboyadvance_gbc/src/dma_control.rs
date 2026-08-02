use std::{cell::RefCell, rc::Rc};

use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use ironboyadvance_sm83::GbSpeed;

use crate::{
    events::{DmaEvent, GbcEvent},
    speed_control::{DOUBLE_SPEED_T_CYCLES, NORMAL_SPEED_T_CYCLES},
};

const OAM_DMA_LENGTH: u16 = 0x00A0;
const OAM_DMA_DESTINATION: u16 = 0xFE00;
const VRAM_DMA_DESTINATION: u16 = 0x8000;
const VRAM_DMA_BLOCK_LENGTH: u8 = 16;

#[derive(Debug, PartialEq, Eq)]
enum VramDmaMode {
    Stopped,
    HdmaPending,
    HdmaActive { block_bytes_remaining: u8 },
    GdmaActive,
}

pub struct DmaTransfer {
    pub source: u16,
    pub destination: u16,
}

pub struct DmaController {
    oam_source: u16,
    oam_index: u16,
    oam_pending: bool,
    oam_active: bool,
    vram_mode: VramDmaMode,
    vram_source: u16,
    vram_destination: u16,
    vram_length: u16,
    speed: GbSpeed,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
}

impl DmaController {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        DmaController {
            oam_source: 0,
            oam_index: 0,
            oam_pending: false,
            oam_active: false,
            vram_mode: VramDmaMode::Stopped,
            vram_source: 0,
            vram_destination: 0,
            vram_length: 0,
            speed: GbSpeed::Normal,
            scheduler,
        }
    }

    pub fn set_speed(&mut self, speed: GbSpeed) {
        self.speed = speed;
    }

    pub fn next_oam_transfer(&self) -> Option<DmaTransfer> {
        match self.oam_active {
            true => Some(DmaTransfer {
                source: self.oam_source.wrapping_add(self.oam_index),
                destination: OAM_DMA_DESTINATION | self.oam_index,
            }),
            false => None,
        }
    }

    pub fn complete_oam_transfer(&mut self, timestamp: usize) {
        self.oam_index += 1;

        match self.oam_index < OAM_DMA_LENGTH {
            true => self.schedule_transfer(timestamp),
            false => self.oam_active = false,
        }
    }

    fn transfer_cycles(&self) -> usize {
        match self.speed {
            GbSpeed::Normal => NORMAL_SPEED_T_CYCLES,
            GbSpeed::Double => DOUBLE_SPEED_T_CYCLES,
        }
    }

    fn schedule_transfer(&mut self, timestamp: usize) {
        self.scheduler
            .borrow_mut()
            .schedule_at_timestamp(GbcEvent::Dma(DmaEvent::OamTransfer), timestamp + self.transfer_cycles());
    }

    fn start_oam_dma(&mut self, value: u8) {
        self.scheduler
            .borrow_mut()
            .cancel_events(GbcEvent::Dma(DmaEvent::OamTransfer));

        self.oam_source = (value as u16) << 8;
        self.oam_index = 0;
        self.oam_pending = true;
        self.oam_active = false;

        let timestamp = self.scheduler.borrow().timestamp();
        self.schedule_transfer(timestamp);
    }

    pub fn activate_oam_dma(&mut self) {
        if self.oam_pending {
            self.oam_pending = false;
            self.oam_active = true;
        }
    }

    pub fn oam_dma_conflict(&self, address: u16) -> bool {
        if !self.oam_active {
            return false;
        }

        let vram_source = matches!(self.oam_source, 0x8000..=0x9FFF);
        match address {
            0x8000..=0x9FFF => vram_source,
            0xFE00..=0xFE9F => true,
            0xFF00..=0xFFFF => false,
            _ => !vram_source,
        }
    }

    pub fn vram_dma_active(&self) -> bool {
        matches!(self.vram_mode, VramDmaMode::GdmaActive | VramDmaMode::HdmaActive { .. })
    }

    pub fn next_vram_transfer(&self) -> Option<DmaTransfer> {
        match self.vram_dma_active() {
            true => Some(DmaTransfer {
                source: self.vram_source,
                destination: VRAM_DMA_DESTINATION | (self.vram_destination & 0x1FFF),
            }),
            false => None,
        }
    }

    pub fn complete_vram_transfer(&mut self) {
        self.vram_source = self.vram_source.wrapping_add(1);
        self.vram_destination = self.vram_destination.wrapping_add(1);
        self.vram_length -= 1;

        if let VramDmaMode::HdmaActive { block_bytes_remaining } = &mut self.vram_mode {
            *block_bytes_remaining -= 1;
            if *block_bytes_remaining == 0 {
                self.vram_mode = VramDmaMode::HdmaPending;
            }
        }

        if self.vram_length == 0 {
            self.vram_mode = VramDmaMode::Stopped;
        }
    }

    pub fn start_h_blank_block(&mut self, cpu_halted: bool) {
        if self.vram_mode == VramDmaMode::HdmaPending && !cpu_halted {
            self.vram_mode = VramDmaMode::HdmaActive {
                block_bytes_remaining: VRAM_DMA_BLOCK_LENGTH,
            };
        }
    }

    pub fn write_vram_dma_control(&mut self, value: u8, in_h_blank: bool) {
        let length = VRAM_DMA_BLOCK_LENGTH as u16 * ((value & 0x7F) + 1) as u16;

        if self.vram_mode != VramDmaMode::Stopped {
            match value & 0x80 == 0 {
                true => self.vram_mode = VramDmaMode::Stopped,
                false => self.vram_length = length,
            }
            return;
        }

        self.vram_length = length;
        self.vram_mode = match value & 0x80 != 0 {
            true => match in_h_blank {
                true => VramDmaMode::HdmaActive {
                    block_bytes_remaining: VRAM_DMA_BLOCK_LENGTH,
                },
                false => VramDmaMode::HdmaPending,
            },
            false => VramDmaMode::GdmaActive,
        };
    }
}

impl SystemMemoryAccess for DmaController {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF46 => (self.oam_source >> 8) as u8,
            0xFF51..=0xFF54 => 0xFF,
            0xFF55 => {
                let remaining = ((self.vram_length / VRAM_DMA_BLOCK_LENGTH as u16) as u8).wrapping_sub(1) & 0x7F;
                let stopped = (self.vram_mode == VramDmaMode::Stopped) as u8;
                remaining | stopped << 7
            }
            _ => panic!("Invalid byte read for DmaController: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF46 => self.start_oam_dma(value),
            0xFF51 => self.vram_source = self.vram_source & 0x00FF | (value as u16) << 8,
            0xFF52 => self.vram_source = self.vram_source & 0xFF00 | (value & 0xF0) as u16,
            0xFF53 => self.vram_destination = self.vram_destination & 0x00FF | ((value & 0x1F) as u16) << 8,
            0xFF54 => self.vram_destination = self.vram_destination & 0xFF00 | (value & 0xF0) as u16,
            _ => panic!("Invalid byte write for DmaController: {:#06X}", address),
        }
    }
}
