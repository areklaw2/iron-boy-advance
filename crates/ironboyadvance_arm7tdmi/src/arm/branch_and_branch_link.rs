use ironboyadvance_utils::bit::BitOps;

use crate::{
    Condition, CpuAction,
    cpu::{Arm7tdmiCpu, LR},
    memory::MemoryInterface,
};

#[derive(Debug, Clone, Copy)]
pub struct BranchAndBranchWithLink {
    value: u32,
}

impl BranchAndBranchWithLink {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        if self.link() {
            cpu.set_register(LR, cpu.pc() - 4)
        }

        let offset = ((self.offset() << 8) as i32) >> 6;
        cpu.set_pc((cpu.pc() as i32).wrapping_add(offset) as u32);
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }

    pub fn disassemble(&self) -> String {
        let cond = self.cond();
        let link = if self.link() { "L" } else { "" };
        let expression = self.offset();
        format!("B{link}{cond} 0x{expression:08X}")
    }

    #[inline]
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }

    #[inline]
    fn link(&self) -> bool {
        self.value.bit(24)
    }

    #[inline]
    fn offset(&self) -> u32 {
        self.value.bits(0..=23)
    }
}
