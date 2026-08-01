use getset::{Getters, MutGetters};
use ironboyadvance_sm83::memory::{InterruptContext, MemoryInterface};

use crate::{events::InterruptEvent, io_registers::IoRegisters};

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct SystemBus {
    io_registers: IoRegisters,
}

impl SystemBus {
    pub fn new() -> Self {
        SystemBus {
            io_registers: IoRegisters::new(),
        }
    }

    pub fn raise_interrupt(&mut self, interrupt_event: InterruptEvent) {
        self.io_registers.interrupt_controller_mut().raise_interrupt(interrupt_event);
    }

    pub fn interrupts_pending(&self) -> bool {
        self.io_registers.interrupt_controller().interrupts_pending()
    }
}

impl MemoryInterface for SystemBus {
    fn load_8(&self, address: u16) -> u8 {
        todo!()
    }

    fn load_16(&self, address: u16) -> u16 {
        todo!()
    }

    fn store_8(&mut self, address: u16, value: u8) {
        todo!()
    }

    fn store_16(&mut self, address: u16, value: u16) {
        todo!()
    }

    fn idle_cycle(&mut self) {
        todo!()
    }

    fn change_speed(&mut self) {
        todo!()
    }

    fn interrupt_context(&self) -> &InterruptContext {
        self.io_registers.interrupt_controller().interrupt_context()
    }

    fn interrupt_context_mut(&mut self) -> &mut InterruptContext {
        self.io_registers.interrupt_controller_mut().interrupt_context_mut()
    }
}
