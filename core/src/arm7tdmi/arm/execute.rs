use crate::{
    arm7tdmi::{arm::ArmInstructionFormat, cpu::Arm7tdmiCpu, CpuAction},
    memory::MemoryInterface,
};

use super::ArmInstruction;

impl<I: MemoryInterface> Arm7tdmiCpu<I> {
    pub fn arm_execute(&mut self, instruction: ArmInstruction) -> CpuAction {
        use ArmInstructionFormat::*;
        match instruction.format {
            BranchAndExchange => self.execute_branch_and_exchange(),
            _ => todo!(),
        }
    }

    pub fn execute_branch_and_exchange(&mut self) -> CpuAction {
        // BX execution switches state
        // When executing an execution if cpu is in thumb it will switch to
        // arm execute arm and then switch back to thumb
        todo!()
    }
}
