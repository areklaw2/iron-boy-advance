use ironboyadvance_utils::bit::BitOps;

use crate::{
    CpuAction,
    cpu::{Arm7tdmiCpu, LR},
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct LongBranchWithLink {
    value: u16,
}

thumb_instruction!(LongBranchWithLink);

impl LongBranchWithLink {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let mut offset = self.offset() as i32;
        match self.high() {
            true => {
                offset <<= 1;
                let temp = (cpu.pc() - 2) | 0b1;
                cpu.set_pc((cpu.register(LR) & !0b1).wrapping_add(offset as u32));
                cpu.set_register(LR, temp);
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => {
                offset = (offset << 21) >> 9;
                cpu.set_register(LR, cpu.pc().wrapping_add(offset as u32));
                CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
            }
        }
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        let high = if self.high() { "hi" } else { "lo" };
        format!("BL #{}({})", offset, high)
    }

    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(0..=10)
    }

    #[inline]
    pub fn high(&self) -> bool {
        self.value.bit(11)
    }
}
