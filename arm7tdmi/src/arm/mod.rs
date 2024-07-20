use bit::BitIndex;
use dissassembler::ArmInstructionFormat;

use crate::{
    dissassembler::{Condition, Register},
    Cpu, Instruction,
};

pub mod dissassembler;
pub mod execute;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ArmInstruction {
    format: ArmInstructionFormat,
    instruction: u32,
    address: u32,
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

impl ArmInstruction {
    pub fn new(format: ArmInstructionFormat, instruction: u32, address: u32) -> ArmInstruction {
        ArmInstruction {
            format,
            instruction,
            address,
        }
    }

    pub fn cond(&self) -> Condition {
        self.instruction.bit_range(28..32).into()
    }

    pub fn rn(&self) -> Register {
        todo!()
    }
}

impl Cpu {
    pub fn arm_execute(&self, instruction: ArmInstruction) {
        todo!("{:?}", instruction)
    }
}

#[cfg(test)]
mod tests {
    use crate::dissassembler::Condition;

    use super::{dissassembler::ArmInstructionFormat, ArmInstruction};
    use ArmInstructionFormat::*;

    #[test]
    fn get_condition() {
        let instruction = ArmInstruction::new(BranchAndExchange, 0x8FFF_FFFF, 0);
        assert_eq!(instruction.cond(), Condition::HI)
    }
}
