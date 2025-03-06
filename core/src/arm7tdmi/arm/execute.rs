use crate::{
    arm7tdmi::{arm::ArmInstructionFormat, cpu::Arm7tdmiCpu, CpuAction, CpuState},
    memory::MemoryInterface,
};

use super::ArmInstruction;

impl<I: MemoryInterface> Arm7tdmiCpu<I> {
    pub fn arm_execute(&mut self, instruction: ArmInstruction) -> CpuAction {
        use ArmInstructionFormat::*;
        match instruction.format {
            BranchAndExchange => self.execute_branch_and_exchange(instruction),
            _ => todo!(),
        }
    }

    pub fn execute_branch_and_exchange(&mut self, instruction: ArmInstruction) -> CpuAction {
        let value = self.get_register(instruction.rn() as usize);
        self.set_cpu_state(CpuState::from_bits((value & 0x1) as u8));
        self.set_pc(value & !0x1);
        self.refill_pipeline();
        CpuAction::PipelineFlush
    }
}
