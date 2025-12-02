use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters, Setters};
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use ironboyadvance_utils::bit::BitOps;
use tracing::debug;

use crate::{
    interrupt_control::{Interrupt, InterruptController},
    scheduler::Scheduler,
    system_control::SystemController,
};

const IE: u32 = 0x04000200;
const IF: u32 = 0x04000202;
pub const WAITCNT: u32 = 0x04000204;
const IME: u32 = 0x04000208;
pub const POSTFLG: u32 = 0x04000300;
pub const HALTCNT: u32 = 0x04000301;
pub const PURPOSE_UNKNOWN: u32 = 0x04000410;
pub const INTERNAL_MEMORY_CONTROL: u32 = 0x04000800;

#[derive(Getters, MutGetters, Setters)]
pub struct IoRegisters {
    scheduler: Rc<RefCell<Scheduler>>,
    interrupt_controller: InterruptController,
    #[getset(get = "pub", get_mut = "pub")]
    system_controller: SystemController,
    data: Vec<u8>,
}

impl IoRegisters {
    pub fn new(scheduler: Rc<RefCell<Scheduler>>) -> Self {
        let interrupt_flags = Rc::new(RefCell::new(Interrupt::from_bits(0)));
        IoRegisters {
            scheduler,
            interrupt_controller: InterruptController::new(interrupt_flags.clone()),
            system_controller: SystemController::new(),
            data: vec![0; 0x400],
        }
    }

    pub fn interrupt_pending(&self) -> bool {
        self.interrupt_controller.interrupt_pending()
    }
}

impl SystemMemoryAccess for IoRegisters {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // IE
            0x04000200 => self.interrupt_controller.read_8(address),
            0x04000201 => self.interrupt_controller.read_8(address),
            // IF
            0x04000202 => self.interrupt_controller.read_8(address),
            0x04000203 => self.interrupt_controller.read_8(address),
            // WAITCNT
            0x04000204 => self.system_controller.read_8(address),
            0x04000205 => self.system_controller.read_8(address),
            // IME
            0x04000208 => self.interrupt_controller.read_8(address),
            // POSTFLG
            0x04000300 => self.system_controller.read_8(address),
            // INTERNAL MEMORY CONTROL
            0x04000800 => self.system_controller.read_8(address),
            0x04000801 => self.system_controller.read_8(address),
            0x04000802 => self.system_controller.read_8(address),
            0x04000803 => self.system_controller.read_8(address),
            _ => {
                debug!("Read byte not in IoRegisters implemented address: {:08X}", address);
                0
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // IE
            0x04000200 => self.interrupt_controller.write_8(address, value),
            0x04000201 => self.interrupt_controller.write_8(address, value),
            // IF
            0x04000202 => self.interrupt_controller.write_8(address, value),
            0x04000203 => self.interrupt_controller.write_8(address, value),
            // WAITCNT
            0x04000204 => self.system_controller.write_8(address, value),
            0x04000205 => self.system_controller.write_8(address, value),
            // IME
            0x04000208 => self.interrupt_controller.write_8(address, value),
            // POSTFLG
            0x04000300 => self.system_controller.write_8(address, value),
            // INTERNAL MEMORY CONTROL
            0x04000800 => self.system_controller.write_8(address, value),
            0x04000801 => self.system_controller.write_8(address, value),
            0x04000802 => self.system_controller.write_8(address, value),
            0x04000803 => self.system_controller.write_8(address, value),
            _ => debug!(
                "Write byte not implemented IoRegisters address: {:08X}, value: {:08X}",
                address, value
            ),
        }
    }
}
