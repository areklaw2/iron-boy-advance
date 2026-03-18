use getset::CopyGetters;

use crate::{
    BitOps, Condition, CpuAction, Exception,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};

#[derive(Debug, Clone, Copy, CopyGetters)]
pub struct SoftwareInterrupt {
    #[getset(get_copy = "pub(crate)")]
    cond: Condition,
    comment: u32,
}

impl SoftwareInterrupt {
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            comment: value.bits(0..=23),
        }
    }
}

impl Instruction for SoftwareInterrupt {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::SoftwareInterrupt);
        CpuAction::PipelineFlush
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let comment = self.comment;
        format!("SWI{} 0x{:08X}", cond, comment)
    }
}
