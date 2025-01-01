use core::memory::MemoryInterface;

use crate::cpu::Cpu;

use super::{disassembler::ArmInstructionFormat, ArmInstruction};

impl<I: MemoryInterface> Cpu<I> {
    pub fn arm_execute(&mut self, instruction: ArmInstruction) {
        use ArmInstructionFormat::*;
        match instruction.format {
            BranchAndExchange => self.execute_branch_and_exchange(),
            _ => todo!(),
        }
    }

    pub fn execute_branch_and_exchange(&mut self) {}
}
