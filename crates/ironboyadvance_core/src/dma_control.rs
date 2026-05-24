use std::{cell::RefCell, rc::Rc};

use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use tracing::debug;

use crate::scheduler::Scheduler;

#[allow(unused)]
pub struct DmaController {
    scheduler: Rc<RefCell<Scheduler>>,
}

impl DmaController {
    pub fn new(scheduler: Rc<RefCell<Scheduler>>) -> Self {
        Self { scheduler }
    }

    pub fn is_active(&self) -> bool {
        false
    }
}

impl SystemMemoryAccess for DmaController {
    fn read_8(&self, address: u32) -> u8 {
        debug!("DMA read not implemented: {:#010X}", address);
        0
    }

    fn write_8(&mut self, address: u32, value: u8) {
        debug!("DMA write not implemented: {:#010X} = {:#04X}", address, value);
    }
}
