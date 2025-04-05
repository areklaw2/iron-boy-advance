//TODO: implement step tests
#[cfg(test)]
use std::fs;

use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::{CpuMode, cpu::Arm7tdmiCpu, psr::ProgramStatusRegister};

mod test_bus;

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
enum Access {
    Nonsequential = 0b0,
    Sequential = 0b1,
    Code = 0b10,
    Dma = 0b100,
    Lock = 0b1000,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Transaction {
    pub kind: TransactionKind,
    pub size: Size,
    pub addr: u32,
    pub data: u32,
    pub cycle: u8,
    pub access: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
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
struct Test {
    #[serde(rename = "initial")]
    initial_state: State,
    #[serde(rename = "final")]
    final_state: State,
    transactions: Vec<Transaction>,
    opcode: u32,
    base_addr: [u32; 1],
}

#[test]
fn single_step_tests() {
    // Will keep a list of the files I want to run until i complete all the instructions
    // completed
    let files = [
        "arm_b_bl.json",
        "arm_bx.json",
        "arm_data_proc_immediate_shift.json",
        "arm_data_proc_immediate.json",
        "arm_data_proc_register_shift.json",
    ];

    //let files = ["arm_data_proc_register_shift.json"];
    for file in files {
        let test_json = fs::read_to_string(format!("../external/arm7tdmi/v1/{file}")).expect("unable to read file");
        let tests: Vec<Test> = serde_json::from_str(&test_json).unwrap();
        for test in tests {
            let mut cpu = Arm7tdmiCpu::new(
                test_bus::TestBus::new(test.base_addr[0], test.opcode, test.transactions.clone()),
                true,
            );

            let intial_state = test.initial_state;
            let final_state = test.final_state;

            cpu.set_general_registers(intial_state.r);
            cpu.set_banked_registers_fiq(intial_state.r_fiq);
            cpu.set_banked_registers_svc(intial_state.r_svc);
            cpu.set_banked_registers_abt(intial_state.r_abt);
            cpu.set_banked_registers_irq(intial_state.r_irq);
            cpu.set_banked_registers_und(intial_state.r_und);
            cpu.set_cpsr(ProgramStatusRegister::from_bits(intial_state.cpsr));
            cpu.set_spsrs(intial_state.spsr.map(|x| ProgramStatusRegister::from_bits(x)));
            cpu.set_pipeline(intial_state.pipeline);

            cpu.cycle();

            assert_eq!(cpu.general_registers(), final_state.r);
            assert_eq!(cpu.banked_registers_fiq(), final_state.r_fiq);
            assert_eq!(cpu.banked_registers_svc(), final_state.r_svc);
            assert_eq!(cpu.banked_registers_abt(), final_state.r_abt);
            assert_eq!(cpu.banked_registers_irq(), final_state.r_irq);
            assert_eq!(cpu.banked_registers_und(), final_state.r_und);
            assert_eq!(cpu.spsrs().map(|x| x.into_bits()), final_state.spsr);
            assert_eq!(cpu.cpsr().into_bits(), final_state.cpsr);
            assert_eq!(cpu.pipeline(), final_state.pipeline);
        }
    }
}
