use std::collections::HashMap;

use crate::memory::{IoMemoryAccess, MemoryAccess, MemoryInterface};

use super::Transaction;

pub struct TestBus {
    data: Vec<u8>,
    base_address: u32,
    opcode: u32,
    transaction_map: HashMap<u32, Transaction>,
}

impl MemoryInterface for TestBus {
    fn load_8(&mut self, address: u32, _access: MemoryAccess, is_instruction: bool) -> u8 {
        self.read_8(address, is_instruction)
    }

    fn load_16(&mut self, address: u32, _access: MemoryAccess, is_instruction: bool) -> u16 {
        self.read_16(address, is_instruction)
    }

    fn load_32(&mut self, address: u32, _access: MemoryAccess, is_instruction: bool) -> u32 {
        self.read_32(address, is_instruction)
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
    fn read_8(&self, address: u32, _is_instruction: bool) -> u8 {
        self.data[address as usize]
    }

    fn read_16(&self, address: u32, is_instruction: bool) -> u16 {
        if !is_instruction {
            return self.transaction_map.get(&address).unwrap().data as u16;
        }

        if address == self.base_address {
            self.opcode as u16
        } else {
            address as u16
        }
    }

    fn read_32(&self, address: u32, is_instruction: bool) -> u32 {
        if !is_instruction {
            return self.transaction_map.get(&address).unwrap().data;
        }

        if address == self.base_address {
            self.opcode
        } else {
            address
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        self.data[address as usize] = value
    }

    fn write_16(&mut self, address: u32, value: u16) {
        assert_eq!(self.transaction_map.get(&address).unwrap().data as u16, value);
    }

    fn write_32(&mut self, address: u32, value: u32) {
        assert_eq!(self.transaction_map.get(&address).unwrap().data, value);
    }
}

impl TestBus {
    pub fn new(base_address: u32, opcode: u32, transactions: &Vec<Transaction>) -> Self {
        let transaction_map: HashMap<u32, Transaction> = transactions.iter().fold(HashMap::new(), |mut map, t| {
            map.insert(t.addr, t.clone());
            map
        });

        TestBus {
            data: vec![0; 0xFFFFFFFF],
            base_address,
            opcode,
            transaction_map,
        }
    }
}
