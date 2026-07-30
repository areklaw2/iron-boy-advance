use crate::{
    CpuAction,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};
use ironboyadvance_common::bits::{BitOps, SignExtend};

#[derive(Debug, Clone, Copy)]
pub struct UnconditionalBranch {
    offset: u16,
}

impl UnconditionalBranch {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            offset: value.bits(0..=10),
        }
    }
}

impl Instruction for UnconditionalBranch {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = self.offset.sign_extend(11) << 1;
        cpu.set_pc(cpu.pc().wrapping_add(offset as u32));
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        format!("B #{}", offset)
    }
}
