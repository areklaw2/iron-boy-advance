use ironboyadvance_common::{memory::SystemMemoryAccess, register_ops::RegisterOps};

use crate::events::InterruptEvent;

pub struct InterruptController {
    interrupt_master_enabled: u32,
    interrupt_enabled: u16,
    interrupt_flags: u16,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            interrupt_master_enabled: 0,
            interrupt_enabled: 0,
            interrupt_flags: 0,
        }
    }

    pub fn interrupt_pending(&self) -> bool {
        (self.interrupt_master_enabled & 0x1 != 0) && ((self.interrupt_flags & self.interrupt_enabled) != 0)
    }

    pub fn raise_interrupt(&mut self, interrupt_event: InterruptEvent) {
        self.interrupt_flags |= 1 << (interrupt_event as u8);
    }
}

impl SystemMemoryAccess for InterruptController {
    type Address = u32;

    fn read_8(&self, address: u32) -> u8 {
        match address {
            // IE
            0x04000200..=0x04000201 => self.interrupt_enabled.read_byte(address),
            // IF
            0x04000202..=0x04000203 => self.interrupt_flags.read_byte(address),
            // IME
            0x04000208..=0x0400020B => self.interrupt_master_enabled.read_byte(address),
            _ => panic!("Invalid byte read for InterruptController: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // IE
            0x04000200..=0x04000201 => self.interrupt_enabled.write_byte(address, value),
            // IF
            0x04000202..=0x04000203 => {
                let shift = ((address & 1) * 8) as u16;
                self.interrupt_flags &= !(u16::from(value) << shift);
            }
            // IME
            0x04000208..=0x0400020B => self.interrupt_master_enabled.write_byte(address, value),
            _ => panic!("Invalid byte write for InterruptController: {:#010X}", address),
        }
    }
}
