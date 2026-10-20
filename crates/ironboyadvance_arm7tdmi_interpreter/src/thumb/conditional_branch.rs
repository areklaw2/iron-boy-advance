use crate::Execute;
use ironboyadvance_arm7tdmi::{CpuAction, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::ConditionalBranch};
use ironboyadvance_common::{bits::SignExtend, memory::MemoryAccess};

impl Execute for ConditionalBranch {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let condition = self.cond();
        if !cpu.is_condition_met(condition) {
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
        } else {
            let offset = self.offset().sign_extend(8) << 1;
            cpu.set_pc(cpu.pc().wrapping_add(offset as u32));
            cpu.pipeline_flush();
            CpuAction::PipelineFlush
        }
    }
}
