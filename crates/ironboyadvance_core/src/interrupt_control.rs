use std::{cell::RefCell, rc::Rc};

use bitfields::bitfield;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::system_bus::{read_reg_16_byte, write_reg_16_byte};

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Interrupt {
    lcd_v_blank: bool,
    lcd_h_blank: bool,
    lcd_v_counter_match: bool,
    timer_0_overflow: bool,
    timer_1_overflow: bool,
    timer_2_overflow: bool,
    timer_3_overflow: bool,
    serial_communication: bool,
    dma_0_overflow: bool,
    dma_1_overflow: bool,
    dma_2_overflow: bool,
    dma_3_overflow: bool,
    keypad: bool,
    gamepak: bool,
    #[bits(2)]
    _reserved: u8,
}

pub struct InterruptController {
    interrupt_master_enable: bool,
    interrupt_enable: Interrupt,
    interrupt_flags: Rc<RefCell<Interrupt>>,
}

impl InterruptController {
    pub fn new(interrupt_flags: Rc<RefCell<Interrupt>>) -> Self {
        InterruptController {
            interrupt_master_enable: false,
            interrupt_enable: Interrupt::from_bits(0),
            interrupt_flags,
        }
    }

    pub fn interrupt_pending(&self) -> bool {
        self.interrupt_master_enable
            && ((self.interrupt_flags.borrow().into_bits() & self.interrupt_enable.into_bits()) != 0)
    }
}

impl SystemMemoryAccess for InterruptController {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // IE
            0x04000200..=0x04000201 => read_reg_16_byte(self.interrupt_enable.into_bits(), address),
            // IF
            0x04000202..=0x04000203 => read_reg_16_byte(self.interrupt_flags.borrow().into_bits(), address),
            // IME
            0x04000208..=0x04000209 => read_reg_16_byte(self.interrupt_master_enable as u16, address),
            _ => panic!("Invalid byte read for InterruptController: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // IE
            0x04000200..=0x04000201 => {
                let new_value = write_reg_16_byte(self.interrupt_enable.into_bits(), address, value);
                self.interrupt_enable.set_bits(new_value);
            }
            // IF
            0x04000202..=0x04000203 => {
                let new_value = write_reg_16_byte(self.interrupt_flags.borrow().into_bits(), address, value);
                self.interrupt_flags.borrow_mut().set_bits(new_value);
            }
            // IME
            0x04000208..=0x04000209 => {
                let new_value = write_reg_16_byte(self.interrupt_master_enable as u16, address, value);
                self.interrupt_master_enable = new_value & 0x1 != 0;
            }
            _ => panic!("Invalid byte write for InterruptController: {:#010X}", address),
        }
    }
}
