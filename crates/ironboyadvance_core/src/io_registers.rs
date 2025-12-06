use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters, Setters};
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use tracing::debug;

use crate::{
    interrupt_control::{Interrupt, InterruptController},
    ppu::Ppu,
    scheduler::Scheduler,
    system_control::SystemController,
};

#[derive(Getters, MutGetters, Setters)]
pub struct IoRegisters {
    ppu: Ppu,
    #[getset(get = "pub", get_mut = "pub")]
    interrupt_controller: InterruptController,
    #[getset(get = "pub", get_mut = "pub")]
    system_controller: SystemController,
    data: Vec<u8>,
}

impl IoRegisters {
    pub fn new(scheduler: Rc<RefCell<Scheduler>>) -> Self {
        let interrupt_flags = Rc::new(RefCell::new(Interrupt::from_bits(0)));
        IoRegisters {
            ppu: Ppu::new(scheduler.clone(), interrupt_flags.clone()),
            interrupt_controller: InterruptController::new(interrupt_flags.clone()),
            system_controller: SystemController::new(),
            data: vec![0; 0x400],
        }
    }
}

impl SystemMemoryAccess for IoRegisters {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // PPU
            0x04000000..=0x04000056 => self.ppu.read_8(address),
            // Interrupt Control
            0x04000200..=0x04000203 | 0x04000208 => self.interrupt_controller.read_8(address),
            // System Control
            0x04000204..=0x04000205 | 0x04000300 => self.system_controller.read_8(address),
            _ => {
                debug!("Read byte not implemented for I/O register: {:#010X}", address);
                0
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // PPU
            0x04000000..=0x04000056 => self.ppu.write_8(address, value),
            // Interrupt Control
            0x04000200..=0x04000203 | 0x04000208 => self.interrupt_controller.write_8(address, value),
            // System Control
            0x04000204..=0x04000205 | 0x04000300..=0x04000301 => self.system_controller.write_8(address, value),
            _ => debug!(
                "Write byte not implemented for I/O register: {:#010X}, value: {:#04X}",
                address, value
            ),
        }
    }
}
