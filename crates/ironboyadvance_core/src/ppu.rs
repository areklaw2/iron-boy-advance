use std::{cell::RefCell, rc::Rc};

use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::{
    interrupt_control::Interrupt,
    io_registers::RegisterOps,
    ppu::registers::{LcdControl, LcdStatus},
    scheduler::Scheduler,
};

pub const CYCLES_PER_PIXEL: u32 = 4;

pub const HDRAW_PIXELS: u32 = 240;
pub const HBLANK_PIXELS: u32 = 68;

pub const HDRAW_CYCLES: u32 = HDRAW_PIXELS * CYCLES_PER_PIXEL;
pub const HBLANK_CYCLES: u32 = HBLANK_PIXELS * CYCLES_PER_PIXEL;
pub const CYCLES_PER_SCANLINE: u32 = HDRAW_CYCLES + HBLANK_CYCLES;

pub const VDRAW_SCANLINES: u32 = 160;
pub const VBLANK_SCANLINES: u32 = 68;
pub const VDRAW_CYCLES: u32 = VDRAW_SCANLINES * CYCLES_PER_SCANLINE;
pub const VBLANK_CYCLES: u32 = VBLANK_SCANLINES * CYCLES_PER_SCANLINE;

pub const CYCLES_PER_FRAME: usize = VDRAW_CYCLES as usize + VBLANK_CYCLES as usize;

mod registers;

pub struct Ppu {
    lcd_control: LcdControl,
    green_swap: bool,
    lcd_status: LcdStatus,
    vertical_counter: u8,
    interrupt_flags: Rc<RefCell<Interrupt>>,
    scheduler: Rc<RefCell<Scheduler>>,
}

impl Ppu {
    pub fn new(scheduler: Rc<RefCell<Scheduler>>, interrupt_flags: Rc<RefCell<Interrupt>>) -> Self {
        Self {
            lcd_control: LcdControl::from_bits(0),
            green_swap: false,
            lcd_status: LcdStatus::from_bits(0),
            vertical_counter: 0,
            interrupt_flags,
            scheduler,
        }
    }
}

impl SystemMemoryAccess for Ppu {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // DISPCNT
            0x04000000..=0x04000001 => self.lcd_control.read_byte(address),
            // Green Swap
            0x04000002 => self.green_swap as u8,
            0x04000003 => 0,
            // DISPSTAT
            0x04000004..=0x04000005 => self.lcd_status.read_byte(address),
            // VCOUNT
            0x04000006..=0x04000007 => (self.vertical_counter as u16).read_byte(address),
            _ => panic!("Invalid byte read for Ppu Register: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // DISPCNT
            0x04000000..=0x04000001 => self.lcd_control.write_byte(address, value),
            // Green Swap
            0x04000002 => self.green_swap = value & 0x1 != 0,
            0x04000003 => {}
            // DISPSTAT
            0x04000004..=0x04000005 => self.lcd_control.write_byte(address, value),
            _ => panic!("Invalid byte write for Ppu Register: {:#010X}", address),
        }
    }
}
