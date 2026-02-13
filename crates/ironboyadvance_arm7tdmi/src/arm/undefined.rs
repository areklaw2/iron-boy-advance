use ironboyadvance_utils::bit::BitOps;

use crate::{Condition, CpuAction, Exception, cpu::Arm7tdmiCpu, memory::MemoryInterface};

#[derive(Debug, Clone, Copy, Default)]
pub struct Undefined {
    value: u32,
}

impl Undefined {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::Undefined);
        CpuAction::PipelineFlush
    }

    pub fn disassemble(&self) -> String {
        "Undefined".into()
    }

    #[inline]
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }
}
