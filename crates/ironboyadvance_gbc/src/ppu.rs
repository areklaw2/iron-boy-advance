use std::{cell::RefCell, rc::Rc};

use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};

use crate::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH, events::GbcEvent};

pub struct Ppu {
    vram: Vec<u8>,
    vram_bank: usize,
    oam: Vec<u8>,
    lcd_registers: Vec<u8>,
    frame_buffer: Vec<u32>,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
}

impl Ppu {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        Ppu {
            vram: vec![0; 0x4000],
            vram_bank: 0,
            oam: vec![0; 0x00A0],
            lcd_registers: vec![0; 0x000C],
            frame_buffer: vec![0xFFFFFFFF; VIEWPORT_WIDTH * VIEWPORT_HEIGHT],
            scheduler,
        }
    }

    pub fn frame_buffer(&self) -> &[u32] {
        &self.frame_buffer
    }

    fn vram_offset(&self, address: u16) -> usize {
        self.vram_bank * 0x2000 | (address as usize & 0x1FFF)
    }
}

impl SystemMemoryAccess for Ppu {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[self.vram_offset(address)],
            0xFE00..=0xFE9F => self.oam[address as usize & 0x9F],
            0xFF40..=0xFF4B => self.lcd_registers[(address - 0xFF40) as usize],
            0xFF4F => self.vram_bank as u8 | 0xFE,
            _ => panic!("Invalid byte read for Ppu: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                let offset = self.vram_offset(address);
                self.vram[offset] = value;
            }
            0xFE00..=0xFE9F => self.oam[address as usize & 0x9F] = value,
            0xFF40..=0xFF4B => self.lcd_registers[(address - 0xFF40) as usize] = value,
            0xFF4F => self.vram_bank = (value & 0x01) as usize,
            _ => panic!("Invalid byte write for Ppu: {:#06X}", address),
        }
    }
}
