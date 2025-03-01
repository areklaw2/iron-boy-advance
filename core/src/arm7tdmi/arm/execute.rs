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
        let mut register = self.get_general_register(instruction.rn() as usize);
        if register & 0x1 != 0 {
            register &= !0x1;
            self.set_cpu_state(CpuState::Thumb);
            self.set_pc(register);
        } else {
            register &= !0x3;
            self.set_cpu_state(CpuState::Arm);
            self.set_pc(register);
        }
        self.refill_pipeline();
        CpuAction::PipelineFlush
    }
}
