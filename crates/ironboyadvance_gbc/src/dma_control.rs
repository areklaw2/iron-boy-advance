use std::{cell::RefCell, rc::Rc};

use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use ironboyadvance_sm83::GbSpeed;

use crate::{
    DOUBLE_SPEED_T_CYCLES, NORMAL_SPEED_T_CYCLES,
    events::{DmaEvent, GbcEvent},
};

const OAM_DMA_LENGTH: u16 = 0x00A0;
const OAM_DMA_DESTINATION: u16 = 0xFE00;

pub struct DmaTransfer {
    pub source: u16,
    pub destination: u16,
}

pub struct DmaController {
    oam_source: u16,
    oam_index: u16,
    oam_active: bool,
    speed: GbSpeed,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
}

impl DmaController {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        DmaController {
            oam_source: 0,
            oam_index: 0,
            oam_active: false,
            speed: GbSpeed::Normal,
            scheduler,
        }
    }

    pub fn set_speed(&mut self, speed: GbSpeed) {
        self.speed = speed;
    }

    pub fn next_transfer(&self) -> Option<DmaTransfer> {
        match self.oam_active {
            true => Some(DmaTransfer {
                source: self.oam_source.wrapping_add(self.oam_index),
                destination: OAM_DMA_DESTINATION | self.oam_index,
            }),
            false => None,
        }
    }

    pub fn complete_transfer(&mut self, timestamp: usize) {
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
        self.oam_source = (value as u16) << 8;
        self.oam_index = 0;
        self.oam_active = true;

        let timestamp = self.scheduler.borrow().timestamp();
        self.schedule_transfer(timestamp);
    }
}

impl SystemMemoryAccess for DmaController {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF46 => (self.oam_source >> 8) as u8,
            _ => panic!("Invalid byte read for DmaController: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF46 => self.start_oam_dma(value),
            _ => panic!("Invalid byte write for DmaController: {:#06X}", address),
        }
    }
}
