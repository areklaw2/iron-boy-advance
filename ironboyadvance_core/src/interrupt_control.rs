use std::{cell::RefCell, rc::Rc};

use bitfields::bitfield;

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

pub struct InterruptControl {
    interrupt_master_enable: bool,
    interrupt_enable: Interrupt,
    interrupt_flags: Rc<RefCell<Interrupt>>,
}

impl InterruptControl {
    pub fn new(interrupt_flags: Rc<RefCell<Interrupt>>) -> Self {
        InterruptControl {
            interrupt_master_enable: false,
            interrupt_enable: Interrupt::from_bits(0),
            interrupt_flags,
        }
    }

    pub fn set_interrupt_master_enable(&mut self, status: bool) {
        self.interrupt_master_enable = status
    }

    pub fn interrupt_master_enable(&self) -> bool {
        self.interrupt_master_enable
    }

    pub fn set_interrupt_enable(&mut self, value: u16) {
        self.interrupt_enable.set_bits(value)
    }

    pub fn interrupt_enable(&self) -> u16 {
        self.interrupt_enable.into_bits()
    }

    pub fn set_interrupt_flags(&mut self, value: u16) {
        self.interrupt_flags.borrow_mut().set_bits(value);
    }

    pub fn interrupt_flags(&self) -> u16 {
        self.interrupt_flags.borrow().into_bits()
    }

    pub fn interrupt_pending(&self) -> bool {
        self.interrupt_master_enable
            && ((self.interrupt_flags.borrow().into_bits() & self.interrupt_enable.into_bits()) != 0)
    }
}
