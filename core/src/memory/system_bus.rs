use std::{cell::RefCell, rc::Rc};

use crate::scheduler::{self, Scheduler};

use super::{IoMemoryAccess, MemoryInterface};

pub struct SystemBus {
    scheduler: Rc<RefCell<Scheduler>>,
    data: Vec<u8>,
}

impl MemoryInterface for SystemBus {
    fn load_8(&self, address: u32) -> u8 {
        self.read_8(address)
    }

    fn load_16(&self, address: u32) -> u16 {
        self.read_16(address)
    }

    fn load_32(&self, address: u32) -> u32 {
        self.read_32(address)
    }

    fn store_8(&mut self, address: u32, value: u8) {
        self.write_8(address, value);
    }

    fn store_16(&mut self, address: u32, value: u16) {
        self.write_16(address, value);
    }

    fn store_32(&mut self, address: u32, value: u32) {
        self.write_32(address, value);
    }
}

impl IoMemoryAccess for SystemBus {
    fn read_8(&self, address: u32) -> u8 {
        todo!()
    }

    fn read_16(&self, address: u32) -> u16 {
        todo!()
    }

    fn read_32(&self, address: u32) -> u32 {
        todo!()
    }

    fn write_8(&self, address: u32, value: u8) {
        todo!()
    }

    fn write_16(&self, address: u32, value: u16) {
        todo!()
    }

    fn write_32(&self, address: u32, value: u32) {
        todo!()
    }
}

impl SystemBus {
    pub fn new(scheduler: Rc<RefCell<Scheduler>>) -> Self {
        SystemBus {
            data: vec![0; 0xFFFFFFFF],
            scheduler,
        }
    }
}
