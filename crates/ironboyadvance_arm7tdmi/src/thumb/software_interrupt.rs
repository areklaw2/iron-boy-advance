use ironboyadvance_utils::bit::BitOps;

use crate::{
    CpuAction, Exception,
    cpu::Arm7tdmiCpu,
    memory::MemoryInterface,
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct SoftwareInterrupt {
    value: u16,
}

thumb_instruction!(SoftwareInterrupt);

impl SoftwareInterrupt {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::SoftwareInterrupt);
        CpuAction::PipelineFlush
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        format!("SWI #{}", offset)
    }

    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(0..=7)
    }
}
