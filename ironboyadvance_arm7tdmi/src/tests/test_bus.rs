use crate::memory::{IoMemoryAccess, MemoryAccess, MemoryInterface, decompose_access_pattern};

use super::{Transaction, TransactionKind};

#[allow(dead_code)]
pub struct TestBus {
    data: Vec<u8>,
    base_address: u32,
    opcode: u32,
    transactions: Vec<Transaction>,
}

impl MemoryInterface for TestBus {
    fn load_8(&mut self, _address: u32, access_pattern: u8) -> u32 {
        let access = decompose_access_pattern(access_pattern);
        let is_instruction_read = access.contains(&MemoryAccess::Instruction);
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

    fn load_16(&mut self, _address: u32, access_pattern: u8) -> u32 {
        let access = decompose_access_pattern(access_pattern);
        let is_instruction_read = access.contains(&MemoryAccess::Instruction);
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

    fn load_32(&mut self, _address: u32, access_pattern: u8) -> u32 {
        let access = decompose_access_pattern(access_pattern);
        let is_instruction_read = access.contains(&MemoryAccess::Instruction);
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

    fn store_8(&mut self, _address: u32, value: u8, _access_pattern: u8) {
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
                assert_eq!(value, transaction.data as u8);
            }
            None => panic!("No transaction found"),
        }
    }

    fn store_16(&mut self, _address: u32, value: u16, _access_pattern: u8) {
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

    fn store_32(&mut self, _address: u32, value: u32, _access_pattern: u8) {
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

    fn idle_cycle(&mut self) {}
}

impl IoMemoryAccess for TestBus {
    fn read_8(&self, address: u32) -> u8 {
        self.data[address as usize]
    }

    fn write_8(&mut self, address: u32, value: u8) {
        self.data[address as usize] = value
    }
}

#[allow(dead_code)]
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
