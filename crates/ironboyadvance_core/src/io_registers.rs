use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters, Setters};
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::{
    interrupt_control::{Interrupt, InterruptControl},
    scheduler::Scheduler,
    system_control::{HaltMode, SystemController},
};

const IE: u32 = 0x04000200;
const IF: u32 = 0x04000202;
const WAITCNT: u32 = 0x04000204;
const IME: u32 = 0x04000208;
const HALTCNT: u32 = 0x04000301;

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
        match address {
            _ => 0, //TODO: add tracing for this
        }
    }

    fn read_16(&self, address: u32) -> u16 {
        match address {
            IE => self.interrupt_control.interrupt_enable(),
            IF => self.interrupt_control.interrupt_flags(),
            WAITCNT => self.system_controller.waitstate_control().into_bits(),
            IME => self.interrupt_control.interrupt_master_enable() as u16,
            _ => 0, //TODO: add tracing for this
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            HALTCNT => match value != 0 {
                true => todo!("Figure out whuy Stopped is ignored"),
                false => {
                    self.system_controller.set_halt_mode(HaltMode::Halted);
                }
            },
            _ => {} //TODO: add tracing for this
        }
    }

    fn write_16(&mut self, address: u32, value: u16) {
        match address {
            IE => self.interrupt_control.set_interrupt_enable(value),
            IF => self.interrupt_control.set_interrupt_flags(value),
            WAITCNT => self.system_controller.set_waitstate_control(value),
            IME => self.interrupt_control.set_interrupt_master_enable(value != 0),
            _ => {} //TODO: add tracing for this
        }
    }
}
