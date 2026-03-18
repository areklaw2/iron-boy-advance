use getset::CopyGetters;

use crate::{
    BitOps, Condition, CpuAction, Exception,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};

#[derive(Debug, Clone, Copy, CopyGetters)]
pub struct Undefined {
    #[getset(get_copy = "pub(crate)")]
    cond: Condition,
}

impl Undefined {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
        }
    }
}

impl Instruction for Undefined {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::Undefined);
        CpuAction::PipelineFlush
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        "Undefined".into()
    }
}
