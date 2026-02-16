use bitfields::bitfield;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::{io_registers::RegisterOps, scheduler::event::InterruptEvent};

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
    game_pak: bool,
    #[bits(2)]
    not_used_14_15: u8,
}

impl RegisterOps<u16> for Interrupt {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct InterruptMasterEnable {
    interrupts_enabled: bool,
    #[bits(31)]
    not_used_1_31: u32,
}

impl RegisterOps<u32> for InterruptMasterEnable {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.set_bits(bits);
    }
}

pub struct InterruptController {
    interrupt_master_enable: InterruptMasterEnable,
    interrupt_enable: Interrupt,
    interrupt_flags: Interrupt,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            interrupt_master_enable: InterruptMasterEnable::from_bits(0),
            interrupt_enable: Interrupt::from_bits(0),
            interrupt_flags: Interrupt::from_bits(0),
        }
    }

    pub fn interrupt_pending(&self) -> bool {
        self.interrupt_master_enable.interrupts_enabled()
            && ((self.interrupt_flags.into_bits() & self.interrupt_enable.into_bits()) != 0)
    }

    pub fn raise_interrupt(&mut self, interrupt_event: InterruptEvent) {
        let interrupt_flag = 1 << (interrupt_event as u8);
        self.interrupt_flags = Interrupt::from_bits(self.interrupt_flags.into_bits() | interrupt_flag)
    }
}

impl SystemMemoryAccess for InterruptController {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // IE
            0x04000200..=0x04000201 => self.interrupt_enable.read_byte(address),
            // IF
            0x04000202..=0x04000203 => self.interrupt_flags.read_byte(address),
            // IME
            0x04000208..=0x0400020B => self.interrupt_master_enable.read_byte(address),
            _ => panic!("Invalid byte read for InterruptController: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // IE
            0x04000200..=0x04000201 => self.interrupt_enable.write_byte(address, value),
            // IF
            0x04000202..=0x04000203 => self.interrupt_flags.write_byte(address, value),
            // IME
            0x04000208..=0x0400020B => self.interrupt_master_enable.write_byte(address, value),
            _ => panic!("Invalid byte write for InterruptController: {:#010X}", address),
        }
    }
}
