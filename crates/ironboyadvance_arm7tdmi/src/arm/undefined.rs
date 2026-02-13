use ironboyadvance_utils::bit::BitOps;

use crate::{Condition, CpuAction, Exception, cpu::Arm7tdmiCpu, memory::MemoryInterface};

#[derive(Debug, Clone)]
pub struct Undefined {
    value: u32,
    executed_pc: u32,
}

impl Undefined {
    #[inline]
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }

    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::Undefined);
        CpuAction::PipelineFlush
    }

    pub fn disassemble(&self) -> String {
        "Undefined".into()
    }
}
