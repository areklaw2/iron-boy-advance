use std::{cell::RefCell, rc::Rc};

use ironboyadvance_arm7tdmi::memory::IoMemoryAccess;

use crate::{scheduler::Scheduler, system_bus::ClockCycleLuts};

pub struct IoRegisters {
    scheduler: Rc<RefCell<Scheduler>>,
    cycle_luts: Rc<RefCell<ClockCycleLuts>>,
    data: Vec<u8>,
}

impl IoRegisters {
    pub fn new(scheduler: Rc<RefCell<Scheduler>>, cycle_luts: Rc<RefCell<ClockCycleLuts>>) -> Self {
        IoRegisters {
            scheduler,
            cycle_luts,
            data: vec![0; 0x400],
        }
    }
}

//TODO: Work on WaitControl
impl IoMemoryAccess for IoRegisters {
    fn read_8(&self, address: u32) -> u8 {
        self.data[address as usize]
    }

    fn write_8(&mut self, address: u32, value: u8) {
        self.data[address as usize] = value
    }
}
