use crate::{arm7tdmi::cpu::Arm7tdmiCpu, memory::MemoryInterface};

use super::{disassembler::ArmInstructionFormat, ArmInstruction};

impl<I: MemoryInterface> Arm7tdmiCpu<I> {
    pub fn arm_execute(&mut self, instruction: ArmInstruction) {
        use ArmInstructionFormat::*;
        match instruction.format {
            BranchAndExchange => self.execute_branch_and_exchange(),
            _ => todo!(),
        }
    }

    pub fn execute_branch_and_exchange(&mut self) {
        // BX execution switches state
        // When executing an execution if cpu is in thumb it will switch to
        // arm execute arm and then switch back to thumb
    }
}
