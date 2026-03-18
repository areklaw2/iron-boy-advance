#[cfg(test)]
mod tests {
    use rayon::prelude::*;
    use serde::Deserialize;
    use serde_repr::Deserialize_repr;
    use std::collections::{HashMap, VecDeque};
    use std::fs;
    use std::path::PathBuf;

    use crate::memory::{MemoryAccess, MemoryInterface, SystemMemoryAccess, decompose_access_pattern};
    use crate::{
        AluOperationsOpcode,
        arm::ArmInstruction,
        cpu::{Arm7tdmiCpu, LastInstruction},
        psr::ProgramStatusRegister,
        thumb::ThumbInstruction,
    };

    #[derive(Debug, Deserialize_repr, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum TransactionKind {
        InstructionRead = 0,
        GeneralRead,
        Write,
    }

    #[derive(Debug, Deserialize_repr, Clone, Copy)]
    #[repr(u8)]
    enum Size {
        Byte = 1,
        HalfWord = 2,
        Word = 4,
    }

    #[derive(Debug)]
    #[allow(unused)]
    enum Access {
        Nonsequential = 0b0,
        Sequential = 0b1,
        Code = 0b10,
        Dma = 0b100,
        Lock = 0b1000,
    }

    #[derive(Debug, Deserialize, Clone, Copy)]
    #[allow(unused)]
    pub struct Transaction {
        kind: TransactionKind,
        size: Size,
        pub addr: u32,
        pub data: u32,
        pub cycle: u8,
        pub access: u8,
    }

    #[derive(Debug, Deserialize)]
    #[allow(non_snake_case, unused)]
    struct State {
        #[serde(rename = "R")]
        r: [u32; 16],
        #[serde(rename = "R_fiq")]
        r_fiq: [u32; 7],
        #[serde(rename = "R_svc")]
        r_svc: [u32; 2],
        #[serde(rename = "R_abt")]
        r_abt: [u32; 2],
        #[serde(rename = "R_irq")]
        r_irq: [u32; 2],
        #[serde(rename = "R_und")]
        r_und: [u32; 2],
        #[serde(rename = "CPSR")]
        cpsr: u32,
        #[serde(rename = "SPSR")]
        spsr: [u32; 5],
        pipeline: [u32; 2],
    }

    #[derive(Debug, Deserialize)]
    #[allow(unused)]
    struct Test {
        #[serde(rename = "initial")]
        initial_state: State,
        #[serde(rename = "final")]
        final_state: State,
        transactions: VecDeque<Transaction>,
        opcode: u32,
        base_addr: [u32; 1],
    }

    #[allow(unused)]
    struct TestBus {
        data: HashMap<u32, u8>,
        base_address: u32,
        opcode: u32,
        transactions: VecDeque<Transaction>,
    }

    impl TestBus {
        fn take_read_transaction(&mut self, access_pattern: u8) -> Transaction {
            let access = decompose_access_pattern(access_pattern);
            let kind = match access.contains(&MemoryAccess::Instruction) {
                true => TransactionKind::InstructionRead,
                false => TransactionKind::GeneralRead,
            };

            let index = self
                .transactions
                .iter()
                .position(|t| t.kind == kind)
                .expect("No transaction found");
            self.transactions.remove(index).unwrap()
        }

        fn take_write_transaction(&mut self) -> Transaction {
            let index = self
                .transactions
                .iter()
                .position(|t| t.kind == TransactionKind::Write)
                .expect("No transaction found");
            self.transactions.remove(index).unwrap()
        }
    }

    impl MemoryInterface for TestBus {
        fn load_8(&mut self, _address: u32, access_pattern: u8) -> u32 {
            self.take_read_transaction(access_pattern).data
        }

        fn load_16(&mut self, _address: u32, access_pattern: u8) -> u32 {
            self.take_read_transaction(access_pattern).data
        }

        fn load_32(&mut self, _address: u32, access_pattern: u8) -> u32 {
            self.take_read_transaction(access_pattern).data
        }

        fn store_8(&mut self, _address: u32, value: u8, _access_pattern: u8) {
            let transaction = self.take_write_transaction();
            assert_eq!(value, transaction.data as u8);
        }

        fn store_16(&mut self, _address: u32, value: u16, _access_pattern: u8) {
            let transaction = self.take_write_transaction();
            assert_eq!(value, transaction.data as u16);
        }

        fn store_32(&mut self, _address: u32, value: u32, _access_pattern: u8) {
            let transaction = self.take_write_transaction();
            assert_eq!(value, transaction.data);
        }

        fn idle_cycle(&mut self) {}
    }

    impl SystemMemoryAccess for TestBus {
        fn read_8(&self, address: u32) -> u8 {
            match self.data.get(&address) {
                Some(value) => *value,
                None => 0,
            }
        }

        fn write_8(&mut self, address: u32, value: u8) {
            self.data.insert(address, value);
        }
    }

    #[allow(dead_code)]
    impl TestBus {
        pub fn new(base_address: u32, opcode: u32, transactions: VecDeque<Transaction>) -> Self {
            TestBus {
                data: HashMap::new(),
                base_address,
                opcode,
                transactions,
            }
        }
    }

    impl Default for TestBus {
        fn default() -> Self {
            Self {
                data: Default::default(),
                base_address: Default::default(),
                opcode: Default::default(),
                transactions: Default::default(),
            }
        }
    }

    fn read_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    fn parse_state(bytes: &[u8], ptr: usize) -> (State, usize) {
        let full_size = read_u32(bytes, ptr) as usize;
        let base = ptr + 8; // skip full_size(4) + pad(4)
        let v: Vec<u32> = (0..40).map(|i| read_u32(bytes, base + i * 4)).collect();
        let state = State {
            r: v[0..16].try_into().unwrap(),
            r_fiq: v[16..23].try_into().unwrap(),
            r_svc: v[23..25].try_into().unwrap(),
            r_abt: v[25..27].try_into().unwrap(),
            r_irq: v[27..29].try_into().unwrap(),
            r_und: v[29..31].try_into().unwrap(),
            cpsr: v[31],
            spsr: v[32..37].try_into().unwrap(),
            pipeline: v[37..39].try_into().unwrap(),
        };
        (state, full_size)
    }

    fn parse_transactions(bytes: &[u8], ptr: usize) -> (VecDeque<Transaction>, usize) {
        let full_size = read_u32(bytes, ptr) as usize;
        let num = read_u32(bytes, ptr + 8) as usize;
        let mut base = ptr + 12;
        let mut transactions = VecDeque::with_capacity(num);
        for _ in 0..num {
            let kind = match read_u32(bytes, base) {
                0 => TransactionKind::InstructionRead,
                1 => TransactionKind::GeneralRead,
                _ => TransactionKind::Write,
            };
            let size = match read_u32(bytes, base + 4) {
                1 => Size::Byte,
                2 => Size::HalfWord,
                _ => Size::Word,
            };
            transactions.push_back(Transaction {
                kind,
                size,
                addr: read_u32(bytes, base + 8),
                data: read_u32(bytes, base + 12),
                cycle: read_u32(bytes, base + 16) as u8,
                access: read_u32(bytes, base + 20) as u8,
            });
            base += 24;
        }
        (transactions, full_size)
    }

    fn parse_opcodes(bytes: &[u8], ptr: usize) -> (u32, usize) {
        let full_size = read_u32(bytes, ptr) as usize;
        // skip full_size(4) + pad(4) + opcode_raw(4) = offset 12
        let base_addr = read_u32(bytes, ptr + 12);
        (base_addr, full_size)
    }

    fn parse_bin_file(path: &PathBuf) -> Result<Vec<Test>, String> {
        let bytes = fs::read(path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
        if read_u32(&bytes, 0) != 0xD33DBAE0 {
            return Err(format!("Bad magic in {:?}", path));
        }
        let num_tests = read_u32(&bytes, 4) as usize;
        let mut ptr = 8usize;
        let mut tests = Vec::with_capacity(num_tests);
        for _ in 0..num_tests {
            let full_size = read_u32(&bytes, ptr) as usize;
            ptr += 4;
            let (initial_state, size) = parse_state(&bytes, ptr);
            ptr += size;
            let (final_state, size) = parse_state(&bytes, ptr);
            ptr += size;
            let (transactions, size) = parse_transactions(&bytes, ptr);
            ptr += size;
            let (base_addr, size) = parse_opcodes(&bytes, ptr);
            ptr += size;
            let _ = full_size; // used for documentation; ptr advances via sub-block sizes
            tests.push(Test {
                initial_state,
                final_state,
                transactions,
                opcode: 0,
                base_addr: [base_addr],
            });
        }
        Ok(tests)
    }

    fn run_test_file(file_path: PathBuf) -> Result<(), String> {
        if file_path.extension().unwrap().to_str().unwrap() != "json" {
            return Ok(());
        }

        // Skip coprocessor instructions
        let skip = ["arm_cdp.json", "arm_mcr_mrc.json", "arm_stc_ldc.json"];
        if skip.contains(&file_path.file_name().unwrap().to_str().unwrap()) {
            return Ok(());
        }

        let bin_path = PathBuf::from(format!("{}.bin", file_path.display()));
        let tests: Vec<Test> = if bin_path.exists() {
            parse_bin_file(&bin_path)?
        } else {
            let test_json = fs::read_to_string(&file_path).map_err(|e| format!("Failed to read {:?}: {}", file_path, e))?;
            serde_json::from_str(&test_json).map_err(|e| format!("Failed to parse {:?}: {}", file_path, e))?
        };

        let mut cpu = Arm7tdmiCpu::new(TestBus::default(), false, true);
        cpu.set_bios_protection(false);

        for test in tests {
            let test_bus = TestBus::new(test.base_addr[0], test.opcode, test.transactions);
            cpu.set_bus(test_bus);

            let initial_state = test.initial_state;
            let final_state = test.final_state;

            cpu.set_general_registers(initial_state.r);
            cpu.set_banked_registers_fiq(initial_state.r_fiq);
            cpu.set_banked_registers_svc(initial_state.r_svc);
            cpu.set_banked_registers_abt(initial_state.r_abt);
            cpu.set_banked_registers_irq(initial_state.r_irq);
            cpu.set_banked_registers_und(initial_state.r_und);
            cpu.set_cpsr(ProgramStatusRegister::from_bits(initial_state.cpsr));
            cpu.set_spsrs(initial_state.spsr.map(|x| ProgramStatusRegister::from_bits(x)));
            cpu.set_pipeline(initial_state.pipeline);

            cpu.cycle();

            assert_eq!(cpu.general_registers(), &final_state.r);
            assert_eq!(cpu.banked_registers_fiq(), &final_state.r_fiq);
            assert_eq!(cpu.banked_registers_svc(), &final_state.r_svc);
            assert_eq!(cpu.banked_registers_abt(), &final_state.r_abt);
            assert_eq!(cpu.banked_registers_irq(), &final_state.r_irq);
            assert_eq!(cpu.banked_registers_und(), &final_state.r_und);
            assert_eq!(
                cpu.spsrs().map(|x| x.into_bits()),
                final_state.spsr.map(|x| ProgramStatusRegister::from_bits(x).into_bits())
            );

            let expected = ProgramStatusRegister::from_bits(final_state.cpsr);

            // The booth multiplication sets the carry. data sheet says its set to a meaningless value. Will ignore result
            let is_mutliply = match cpu.last_instruction() {
                Some(LastInstruction::Arm(ArmInstruction::Multiply(_) | ArmInstruction::MultiplyLong(_))) => true,
                Some(LastInstruction::Thumb(ThumbInstruction::AluOperations(operation))) => {
                    AluOperationsOpcode::from(operation.opcode()) == AluOperationsOpcode::MUL
                }
                _ => false,
            };

            let actual = cpu.cpsr_mut();
            if is_mutliply {
                actual.set_carry(expected.carry());
            }
            assert_eq!(expected.into_bits(), actual.into_bits());

            assert_eq!(cpu.pipeline(), &final_state.pipeline);
        }

        Ok(())
    }

    #[test]
    fn single_step_tests() {
        let directory = fs::read_dir("../../external/arm7tdmi/v1").expect("Unable to access directory");
        let file_paths: Vec<PathBuf> = directory.filter_map(|entry| entry.ok()).map(|entry| entry.path()).collect();

        // Process files in parallel
        file_paths
            .par_iter()
            .try_for_each(|path| {
                run_test_file(path.clone()).map_err(|e| {
                    eprintln!("Test failed in file {:?}: {}", path, e);
                    e
                })
            })
            .expect("Test failures occurred");
    }
}
