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
        let mut value = self.get_general_register(instruction.rn() as usize);
        if value & 0x1 != 0 {
            value &= !0x1;
            self.set_cpu_state(CpuState::Thumb);
            self.set_pc(value);
        } else {
            value &= !0x3;
            self.set_cpu_state(CpuState::Arm);
            self.set_pc(value);
        }
        self.refill_pipeline();
        CpuAction::PipelineFlush
    }
}
