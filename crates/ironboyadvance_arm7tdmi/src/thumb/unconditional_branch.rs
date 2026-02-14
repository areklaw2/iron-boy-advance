use ironboyadvance_utils::bit::BitOps;

use crate::{CpuAction, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::thumb_instruction};

#[derive(Debug, Clone, Copy)]
pub struct UnconditionalBranch {
    value: u16,
}

thumb_instruction!(UnconditionalBranch);

impl UnconditionalBranch {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = (((self.offset() as u32) << 21) as i32) >> 20;
        cpu.set_pc(cpu.pc().wrapping_add(offset as u32));
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        format!("B #{}", offset)
    }

    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(0..=10)
    }
}
