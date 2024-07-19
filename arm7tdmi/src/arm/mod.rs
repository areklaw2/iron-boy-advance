use dissassembler::ArmInstructionFormat;

use crate::{Cpu, Instruction};

pub mod dissassembler;
pub mod execute;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ArmInstruction {
    format: ArmInstructionFormat,
    instruction: u32,
    address: u32,
}

impl ArmInstruction {
    pub fn new(format: ArmInstructionFormat, instruction: u32, address: u32) -> ArmInstruction {
        ArmInstruction {
            format,
            instruction,
            address,
        }
    }
}

impl Instruction for ArmInstruction {
    type Size = u32;

    fn decode(instruction: u32, address: u32) -> ArmInstruction {
        ArmInstruction {
            format: instruction.into(),
            instruction,
            address,
        }
    }

    fn value(&self) -> u32 {
        self.instruction
    }
}

impl Cpu {
    pub fn arm_execute(&self, instruction: ArmInstruction) {
        todo!("{:?}", instruction)
    }
}
