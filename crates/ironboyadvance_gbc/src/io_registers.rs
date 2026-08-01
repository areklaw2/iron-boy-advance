use getset::{Getters, MutGetters};
use ironboyadvance_common::memory::SystemMemoryAccess;
use tracing::debug;

use crate::interrupt_control::InterruptController;

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct IoRegisters {
    interrupt_controller: InterruptController,
}

impl IoRegisters {
    pub fn new() -> Self {
        IoRegisters {
            interrupt_controller: InterruptController::new(),
        }
    }
}

impl SystemMemoryAccess for IoRegisters {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            // Interrupt Control
            0xFF0F | 0xFFFF => self.interrupt_controller.read_8(address),
            _ => {
                debug!("Read byte not implemented for I/O register: {:#06X}", address);
                0xFF
            }
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            // Interrupt Control
            0xFF0F | 0xFFFF => self.interrupt_controller.write_8(address, value),
            _ => debug!("Write byte not implemented for I/O register: {:#06X}", address),
        }
    }
}
