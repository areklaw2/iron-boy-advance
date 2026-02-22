use crate::BitOps;

use crate::{
    Condition, CpuAction,
    cpu::Arm7tdmiCpu,
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct ConditionalBranch {
    value: u16,
}

thumb_instruction!(ConditionalBranch);

impl ConditionalBranch {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let condition = self.cond();
        if !cpu.is_condition_met(condition) {
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
        } else {
            let offset = (((self.offset() as u32) << 24) as i32) >> 23;
            cpu.set_pc(cpu.pc().wrapping_add(offset as u32));
            cpu.pipeline_flush();
            CpuAction::PipelineFlush
        }
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let offset = self.offset();
        format!("B{} #{}", cond, offset)
    }

    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(0..=7)
    }

    #[inline]
    pub fn cond(&self) -> Condition {
        (self.value.bits(8..=11) as u32).into()
    }
}
