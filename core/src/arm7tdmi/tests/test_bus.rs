use crate::memory::{IoMemoryAccess, MemoryAccess, MemoryInterface};

use super::Transaction;

pub struct TestBus {
    data: Vec<u8>,
    transactions: Vec<Transaction>,
}

impl MemoryInterface for TestBus {
    fn load_8(&mut self, address: u32, _access: MemoryAccess) -> u8 {
        self.read_8(address)
    }

    fn load_16(&mut self, address: u32, _access: MemoryAccess) -> u16 {
        self.read_16(address)
    }

    fn load_32(&mut self, address: u32, _access: MemoryAccess) -> u32 {
        self.read_32(address)
    }

    fn store_8(&mut self, address: u32, value: u8, _access: MemoryAccess) {
        self.write_8(address, value);
    }

    fn store_16(&mut self, address: u32, value: u16, _access: MemoryAccess) {
        self.write_16(address, value);
    }

    fn store_32(&mut self, address: u32, value: u32, _access: MemoryAccess) {
        self.write_32(address, value);
    }
}

impl IoMemoryAccess for TestBus {
    fn read_8(&self, address: u32) -> u8 {
        self.data[address as usize]
    }

    fn write_8(&mut self, address: u32, value: u8) {
        self.data[address as usize] = value
    }
}

impl TestBus {
    pub fn new(transactions: Vec<Transaction>) -> Self {
        TestBus {
            data: vec![0; 0xFFFFFFFF],
            transactions,
        }
    }
}
