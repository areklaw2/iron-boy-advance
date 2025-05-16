//TODO: implement step tests

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_repr::Deserialize_repr;
    use std::fs;

    use crate::memory::{IoMemoryAccess, MemoryAccess, MemoryInterface, decompose_access_pattern};
    use crate::{cpu::Arm7tdmiCpu, psr::ProgramStatusRegister};

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
        transactions: Vec<Transaction>,
        opcode: u32,
        base_addr: [u32; 1],
    }

    #[allow(unused)]
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
                } else if !is_instruction_read && transaction.kind == TransactionKind::GeneralRead {
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
                } else if !is_instruction_read && transaction.kind == TransactionKind::GeneralRead {
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
                } else if !is_instruction_read && transaction.kind == TransactionKind::GeneralRead {
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

    #[test]
    fn single_step_tests() {
        // Will keep a list of the files I want to run until i complete all the instructions
        // completed
        let files = [
            // "arm_b_bl.json",                      //Done
            // "arm_bx.json",                        //Done
            // "arm_cdp.json",                       //Skip
            // "arm_data_proc_immediate.json",       //Done
            // "arm_data_proc_immediate_shift.json", //Done
            // "arm_data_proc_register_shift.json",  //Done
            // "arm_ldm_stm.json",                   //Done
            // "arm_ldr_str_immediate_offset.json",  //Done
            // "arm_ldr_str_register_offset.json",   //Done
            // "arm_ldrh_strh.json",                 //Done
            // "arm_ldrsb_ldrsh.json",               //Done
            // "arm_mcr_mrc.json",                   //Skip
            // "arm_mrs.json",                       //Done
            // "arm_msr_imm.json",                   //Done
            // "arm_msr_reg.json",                   //Done
            // "arm_mul_mla.json",                   //Done
            // "arm_mull_mlal.json",                 //Done
            // "arm_stc_ldc.json",                   //Skip
            // "arm_swi.json",                       //Done
            // "arm_swp.json",                       //Done
            // "thumb_add_cmp_mov_hi.json",          //Done
            // "thumb_add_sp_or_pc.json",            //Done
            // "thumb_add_sub.json",                 //Done
            // "thumb_add_sub_sp.json", //Done
            // "thumb_b.json",
            // "thumb_bcc.json",
            // "thumb_bl_blx_prefix.json",
            // "thumb_bl_suffix.json",
            // "thumb_bx.json",                     //Done
            // "thumb_data_proc.json",              //Done
            // "thumb_ldm_stm.json",                //Done
            // "thumb_ldr_pc_rel.json",             //Done
            // "thumb_ldr_str_imm_offset.json",     //Done
            // "thumb_ldr_str_reg_offset.json",     //Done
            // "thumb_ldr_str_sp_rel.json",         //Done
            // "thumb_ldrb_strb_imm_offset.json",   //Done
            // "thumb_ldrh_strh_imm_offset.json",   //Done
            // "thumb_ldrh_strh_reg_offset.json",   //Done
            // "thumb_ldrsb_strb_reg_offset.json",  //Done
            // "thumb_ldrsh_ldrsb_reg_offset.json", //Done
            // "thumb_lsl_lsr_asr.json",            //Done
            // "thumb_mov_cmp_add_sub.json",        //Done
            // "thumb_push_pop.json",               //Done
            // "thumb_swi.json",
            // "thumb_undefined_bcc.json",
        ];

        // let directory = fs::read_dir("../external/arm7tdmi/v1").expect("Unable to access directory");
        // for file in directory {
        //     let file = file.unwrap().path();
        // }

        let skip = ["arm_cdp.json", "arm_mcr_mrc.json", "arm_stc_ldc.json"];

        let mut cpu = Arm7tdmiCpu::new(TestBus::default(), true);
        for file in files {
            if skip.contains(&file) {
                continue;
            }

            let test_json = fs::read_to_string(format!("../external/arm7tdmi/v1/{file}")).expect("unable to read file");
            let tests: Vec<Test> = serde_json::from_str(&test_json).unwrap();
            for test in tests {
                let test_bus = TestBus::new(test.base_addr[0], test.opcode, test.transactions.clone());
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

                assert_eq!(cpu.general_registers(), final_state.r);
                assert_eq!(cpu.banked_registers_fiq(), final_state.r_fiq);
                assert_eq!(cpu.banked_registers_svc(), final_state.r_svc);
                assert_eq!(cpu.banked_registers_abt(), final_state.r_abt);
                assert_eq!(cpu.banked_registers_irq(), final_state.r_irq);
                assert_eq!(cpu.banked_registers_und(), final_state.r_und);
                assert_eq!(
                    cpu.spsrs().map(|x| x.into_bits()),
                    final_state.spsr.map(|x| ProgramStatusRegister::from_bits(x).into_bits())
                );

                let expected = ProgramStatusRegister::from_bits(final_state.cpsr);
                let mut actual = cpu.cpsr();
                // the booth multiplication sets the carry. data sheet says its set to a meaningless value. Will ignore result
                if ["MUL", "MLA", "MULL", "MLAL"]
                    .iter()
                    .any(|s| cpu.dissassembled_instruction.contains(s))
                {
                    actual.set_carry(expected.carry());
                }
                assert_eq!(expected.into_bits(), actual.into_bits());

                assert_eq!(cpu.pipeline(), final_state.pipeline);

                cpu.reset(true)
            }
        }
    }
}
