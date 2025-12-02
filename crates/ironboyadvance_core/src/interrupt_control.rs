use std::{cell::RefCell, rc::Rc};

use bitfields::bitfield;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

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
            //IE
            0x04000200 => (self.interrupt_enable.into_bits() & 0xFF) as u8,
            0x04000201 => (self.interrupt_enable.into_bits() >> 8) as u8,
            //IF
            0x04000202 => (self.interrupt_flags.borrow().into_bits() & 0xFF) as u8,
            0x04000203 => (self.interrupt_flags.borrow().into_bits() >> 8) as u8,
            // IME
            0x04000208 => self.interrupt_master_enable as u8,
            _ => {
                panic!("Read byte not in IoRegisters implemented address: {:08X}", address)
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            //IE
            0x04000200 => {
                let current = self.interrupt_enable.into_bits();
                self.interrupt_enable.set_bits((current & 0xFF00) | value as u16);
            }
            0x04000201 => {
                let current = self.interrupt_enable.into_bits();
                self.interrupt_enable.set_bits((current & 0x00FF) | ((value as u16) << 8));
            }
            //IF
            0x04000202 => {
                let current = self.interrupt_flags.borrow().into_bits();
                self.interrupt_flags.borrow_mut().set_bits((current & 0xFF00) | value as u16);
            }
            0x04000203 => {
                let current = self.interrupt_flags.borrow().into_bits();
                self.interrupt_flags
                    .borrow_mut()
                    .set_bits((current & 0x00FF) | ((value as u16) << 8));
            }
            // IME
            0x04000208 => self.interrupt_master_enable = value & 0x1 != 0,
            _ => {
                panic!("Write byte not in IoRegisters implemented address: {:08X}", address)
            }
        }
    }
}
