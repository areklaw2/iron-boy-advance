use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    arm::BranchAndBranchWithLink,
    cpu::{Arm7tdmiCpu, LR},
    memory::MemoryInterface,
};
use ironboyadvance_common::bits::SignExtend;

impl Execute for BranchAndBranchWithLink {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        if self.link() {
            cpu.set_register(LR, cpu.pc() - 4)
        }

        let offset = self.offset().sign_extend(24) << 2;
        cpu.set_pc((cpu.pc() as i32).wrapping_add(offset) as u32);
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }
}
