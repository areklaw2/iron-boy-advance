use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters};
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use tracing::debug;

use crate::{
    events::GbcEvent, interrupt_control::InterruptController, serial_transfer::SerialTransfer,
    speed_control::SpeedController,
};

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct IoRegisters {
    interrupt_controller: InterruptController,
    serial_transfer: SerialTransfer,
    speed_controller: SpeedController,
}

impl IoRegisters {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        IoRegisters {
            interrupt_controller: InterruptController::new(),
            serial_transfer: SerialTransfer::new(scheduler),
            speed_controller: SpeedController::new(),
        }
    }
}

impl SystemMemoryAccess for IoRegisters {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            // Serial Transfer
            0xFF01..=0xFF02 => self.serial_transfer.read_8(address),
            // Interrupt Control
            0xFF0F | 0xFFFF => self.interrupt_controller.read_8(address),
            // Speed Control
            0xFF4D => self.speed_controller.read_8(address),
            _ => {
                debug!("Read byte not implemented for I/O register: {:#06X}", address);
                0xFF
            }
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            // Serial Transfer
            0xFF01..=0xFF02 => self.serial_transfer.write_8(address, value),
            // Interrupt Control
            0xFF0F | 0xFFFF => self.interrupt_controller.write_8(address, value),
            // Speed Control
            0xFF4D => self.speed_controller.write_8(address, value),
            _ => debug!("Write byte not implemented for I/O register: {:#06X}", address),
        }
    }
}
