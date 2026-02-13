use ironboyadvance_utils::bit::BitOps;

use crate::{Condition, CpuAction, Exception, cpu::Arm7tdmiCpu, memory::MemoryInterface};

#[derive(Debug, Clone)]
pub struct SoftwareInterrupt {
    value: u32,
    executed_pc: u32,
}

impl SoftwareInterrupt {
    #[inline]
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }

    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::SoftwareInterrupt);
        CpuAction::PipelineFlush
    }

    pub fn disassemble(&self) -> String {
        let cond = self.cond();
        let comment = self.comment();
        format!("SWI{} 0x{:08X}", cond, comment)
    }

    #[inline]
    pub fn comment(&self) -> u32 {
        self.value.bits(0..=23)
    }
}
