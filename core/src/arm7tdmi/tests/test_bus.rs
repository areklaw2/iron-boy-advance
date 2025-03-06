use std::collections::HashMap;

use crate::memory::{decompose_memory_access, IoMemoryAccess, MemoryAccess, MemoryAccessKind, MemoryInterface};

use super::{Transaction, TransactionKind};

pub struct TestBus {
    data: Vec<u8>,
    base_address: u32,
    opcode: u32,
    transactions: Vec<Transaction>,
}

impl MemoryInterface for TestBus {
    fn load_8(&mut self, address: u32, _access: MemoryAccess) -> u32 {
        self.read_8(address) as u32
    }

    fn load_16(&mut self, _address: u32, access: MemoryAccess) -> u32 {
        let access = decompose_memory_access(access);
        let is_instruction_read = access.contains(&MemoryAccessKind::Instruction);
        let mut transaction_index = None;
        for (i, transaction) in self.transactions.iter().enumerate() {
            if is_instruction_read && transaction.kind == TransactionKind::InstructionRead {
                transaction_index = Some(i);
                break;
            } else if transaction.kind == TransactionKind::GeneralRead {
                transaction_index = Some(i);
                break;
            }
        }

        match transaction_index {
            Some(index) => {
                let transaction = self.transactions.remove(index);
                transaction.data
            }
            None => panic!("No transaction found"),
        }
    }

    fn load_32(&mut self, _address: u32, access: MemoryAccess) -> u32 {
        let access = decompose_memory_access(access);
        let is_instruction_read = access.contains(&MemoryAccessKind::Instruction);
        let mut transaction_index = None;
        for (i, transaction) in self.transactions.iter().enumerate() {
            if is_instruction_read && transaction.kind == TransactionKind::InstructionRead {
                transaction_index = Some(i);
                break;
            } else if transaction.kind == TransactionKind::GeneralRead {
                transaction_index = Some(i);
                break;
            }
        }

        match transaction_index {
            Some(index) => {
                let transaction = self.transactions.remove(index);
                transaction.data
            }
            None => panic!("No transaction found"),
        }
    }

    fn store_8(&mut self, address: u32, value: u8, _access: MemoryAccess) {
        self.write_8(address, value);
    }

    fn store_16(&mut self, _address: u32, value: u16, _access: MemoryAccess) {
        let mut transaction_index = None;
        for (i, transaction) in self.transactions.iter().enumerate() {
            if transaction.kind == TransactionKind::Write {
                transaction_index = Some(i);
                break;
            }
        }

        match transaction_index {
            Some(index) => {
                let transaction = self.transactions.remove(index);
                assert_eq!(value, transaction.data as u16);
            }
            None => panic!("No transaction found"),
        }
    }

    fn store_32(&mut self, _address: u32, value: u32, _access: MemoryAccess) {
        let mut transaction_index = None;
        for (i, transaction) in self.transactions.iter().enumerate() {
            if transaction.kind == TransactionKind::Write {
                transaction_index = Some(i);
                break;
            }
        }

        match transaction_index {
            Some(index) => {
                let transaction = self.transactions.remove(index);
                assert_eq!(value, transaction.data);
            }
            None => panic!("No transaction found"),
        }
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
    pub fn new(base_address: u32, opcode: u32, transactions: Vec<Transaction>) -> Self {
        TestBus {
            data: vec![0; 0xFFFFFFFF],
            base_address,
            opcode,
            transactions,
        }
    }
}
