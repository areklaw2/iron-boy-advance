use bit::BitIndex;
use dissassembler::ArmInstructionFormat;

use crate::{
    dissassembler::{Condition, Register},
    Cpu, Instruction,
};

pub mod dissassembler;
pub mod execute;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmInstruction {
    format: ArmInstructionFormat,
    value: u32,
    address: u32,
}

impl Instruction for ArmInstruction {
    type Size = u32;

    fn decode(instruction: u32, address: u32) -> ArmInstruction {
        ArmInstruction {
            format: instruction.into(),
            value: instruction,
            address,
        }
    }

    fn disassable(&self) -> String {
        use ArmInstructionFormat::*;
        match self.format {
            BranchAndExchange => self.disassemble_branch_and_exchange(),
            _ => todo!(),
        }
    }

    fn value(&self) -> u32 {
        self.value
    }
}

impl ArmInstruction {
    pub fn new(format: ArmInstructionFormat, instruction: u32, address: u32) -> ArmInstruction {
        ArmInstruction {
            format,
            value: instruction,
            address,
        }
    }

    pub fn cond(&self) -> Condition {
        self.value.bit_range(28..32).into()
    }

    pub fn rn(&self) -> Register {
        use ArmInstructionFormat::*;
        match self.format {
            BranchAndExchange => self.value.bit_range(0..4).into(),
            _ => todo!(),
        }
    }
}

impl Cpu {
    pub fn arm_decode_and_execute(&mut self, instruction: u32, pc: u32) {
        let instruction = ArmInstruction::decode(instruction, pc);

        todo!("{:?}", instruction)
    }
}

#[cfg(test)]
mod tests {
    use crate::dissassembler::{Condition, Register};

    use super::{dissassembler::ArmInstructionFormat, ArmInstruction};
    use ArmInstructionFormat::*;

    #[test]
    fn get_condition() {
        let instruction = ArmInstruction::new(BranchAndExchange, 0x8FFF_FFFF, 0);
        assert_eq!(instruction.cond(), Condition::HI)
    }

    #[test]
    fn get_rn() {
        let instruction = ArmInstruction::new(BranchAndExchange, 0x8FFF_FFFC, 0);
        assert_eq!(instruction.rn(), Register::R12)
    }
}
