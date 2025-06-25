use std::{cell::RefCell, rc::Rc};

use bitfields::bitfield;

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Interrupt {}

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
}
