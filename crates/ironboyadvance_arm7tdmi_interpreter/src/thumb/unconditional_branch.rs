use crate::Execute;
use ironboyadvance_arm7tdmi::{CpuAction, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::UnconditionalBranch};
use ironboyadvance_common::bits::SignExtend;

impl Execute for UnconditionalBranch {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = self.offset().sign_extend(11) << 1;
        cpu.set_pc(cpu.pc().wrapping_add(offset as u32));
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }
}
