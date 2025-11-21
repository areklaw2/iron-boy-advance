use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters, Setters};
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use tracing::debug;

use crate::{
    interrupt_control::{Interrupt, InterruptControl},
    scheduler::Scheduler,
    system_control::SystemController,
};

const IE: u32 = 0x04000200;
const IF: u32 = 0x04000202;
pub const WAITCNT: u32 = 0x04000204;
const IME: u32 = 0x04000208;
pub const POSTFLG: u32 = 0x04000300;
pub const HALTCNT: u32 = 0x04000301;

#[derive(Getters, MutGetters, Setters)]
pub struct IoRegisters {
    scheduler: Rc<RefCell<Scheduler>>,
    interrupt_control: InterruptControl,
    #[getset(get = "pub", get_mut = "pub")]
    system_controller: SystemController,
    data: Vec<u8>,
}

impl IoRegisters {
    pub fn new(scheduler: Rc<RefCell<Scheduler>>) -> Self {
        let interrupt_flags = Rc::new(RefCell::new(Interrupt::from_bits(0)));
        IoRegisters {
            scheduler,
            interrupt_control: InterruptControl::new(interrupt_flags.clone()),
            system_controller: SystemController::new(),
            data: vec![0; 0x400],
        }
    }

    pub fn interrupt_pending(&self) -> bool {
        self.interrupt_control.interrupt_pending()
    }
}

impl SystemMemoryAccess for IoRegisters {
    fn read_8(&self, address: u32) -> u8 {
        let value = self.read_16(address & !0x1);
        match address & 0x1 != 0 {
            true => (value >> 8) as u8,
            false => value as u8,
        }
    }

    fn read_16(&self, address: u32) -> u16 {
        match address {
            IE => self.interrupt_control.interrupt_enable(),
            IF => self.interrupt_control.interrupt_flags(),
            WAITCNT => self.system_controller.read_16(address),
            IME => self.interrupt_control.interrupt_master_enable() as u16,
            POSTFLG => self.system_controller.read_16(address),
            _ => {
                debug!("Read not implemented address: {:08X}", address);
                0
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        let current_value = self.read_16(address & !0x1);
        let new_value = match address & 0x1 != 0 {
            true => (current_value & 0x00FF) | ((value as u16) << 8),
            false => (current_value & 0xFF00) | value as u16,
        };
        self.write_16(address & !0x1, new_value);
    }

    fn write_16(&mut self, address: u32, value: u16) {
        match address {
            IE => self.interrupt_control.set_interrupt_enable(value),
            IF => self.interrupt_control.set_interrupt_flags(value),
            WAITCNT => self.system_controller.write_16(address, value),
            IME => self.interrupt_control.set_interrupt_master_enable(value != 0),
            POSTFLG => self.system_controller.write_16(address, value),
            _ => debug!("Write not implemented address: {:08X}, value: {:08X}", address, value),
        }
    }
}
