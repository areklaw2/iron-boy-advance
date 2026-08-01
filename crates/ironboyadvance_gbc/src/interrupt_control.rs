use ironboyadvance_common::memory::SystemMemoryAccess;
use ironboyadvance_sm83::memory::InterruptContext;

use crate::events::InterruptEvent;

const INTERRUPT_MASK: u8 = 0x1F;
const INTERRUPT_FLAGS_UNUSED_BITS: u8 = 0xE0;

pub struct InterruptController {
    interrupt_context: InterruptContext,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            interrupt_context: InterruptContext::default(),
        }
    }

    pub fn interrupt_context(&self) -> &InterruptContext {
        &self.interrupt_context
    }

    pub fn interrupt_context_mut(&mut self) -> &mut InterruptContext {
        &mut self.interrupt_context
    }

    pub fn interrupts_pending(&self) -> bool {
        self.interrupt_context.pending_interrupt() != 0
    }

    pub fn raise_interrupt(&mut self, interrupt_event: InterruptEvent) {
        self.interrupt_context.raise_interrupt(interrupt_event as u8);
    }
}

impl SystemMemoryAccess for InterruptController {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF0F => self.interrupt_context.interrupt_flags() | INTERRUPT_FLAGS_UNUSED_BITS,
            0xFFFF => self.interrupt_context.interrupt_enabled(),
            _ => panic!("Invalid byte read for InterruptController: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF0F => self.interrupt_context.set_interrupt_flags(value & INTERRUPT_MASK),
            0xFFFF => self.interrupt_context.set_interrupt_enabled(value),
            _ => panic!("Invalid byte write for InterruptController: {:#06X}", address),
        };
    }
}
