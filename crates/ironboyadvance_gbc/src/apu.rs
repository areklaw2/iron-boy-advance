use std::{cell::RefCell, rc::Rc};

use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};

use crate::events::GbcEvent;

pub struct Apu {
    audio_registers: Vec<u8>,
    audio_buffer: Vec<(f32, f32)>,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
}

impl Apu {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        Apu {
            audio_registers: vec![0; (0xFF3F - 0xFF10 + 1) as usize],
            audio_buffer: Vec::new(),
            scheduler,
        }
    }

    pub fn audio_buffer(&self) -> &[(f32, f32)] {
        &self.audio_buffer
    }

    pub fn clear_audio_buffer(&mut self) {
        self.audio_buffer.clear();
    }
}

impl SystemMemoryAccess for Apu {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF10..=0xFF3F => self.audio_registers[(address - 0xFF10) as usize],
            _ => panic!("Invalid byte read for Apu: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF10..=0xFF3F => self.audio_registers[(address - 0xFF10) as usize] = value,
            _ => panic!("Invalid byte write for Apu: {:#06X}", address),
        }
    }
}
