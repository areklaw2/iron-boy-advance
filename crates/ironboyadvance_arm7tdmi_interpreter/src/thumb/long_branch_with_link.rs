use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    cpu::{Arm7tdmiCpu, LR},
    memory::MemoryInterface,
    thumb::LongBranchWithLink,
};
use ironboyadvance_common::{bits::SignExtend, memory::MemoryAccess};

impl Execute for LongBranchWithLink {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match self.high() {
            true => {
                let offset = (self.offset() as u32) << 1;
                let temp = (cpu.pc() - 2) | 0b1;
                cpu.set_pc((cpu.register(LR) & !0b1).wrapping_add(offset));
                cpu.set_register(LR, temp);
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => {
                let offset = self.offset().sign_extend(11) << 12;
                cpu.set_register(LR, cpu.pc().wrapping_add(offset as u32));
                CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
            }
        }
    }
}
