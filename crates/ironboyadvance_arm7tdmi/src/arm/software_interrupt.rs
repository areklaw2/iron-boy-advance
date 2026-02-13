use core::fmt;

use ironboyadvance_utils::bit::BitOps;

use crate::{Condition, CpuAction, Exception, cpu::Arm7tdmiCpu, memory::MemoryInterface};

#[derive(Debug, Clone, Copy)]
pub struct SoftwareInterrupt {
    value: u32,
}

impl SoftwareInterrupt {
    pub fn new(value: u32) -> Self {
        Self { value }
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
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }

    #[inline]
    pub fn comment(&self) -> u32 {
        self.value.bits(0..=23)
    }
}

impl fmt::Display for SoftwareInterrupt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = "SoftwareInterrupt";
        write!(
            f,
            "ArmInstruction: name: {:?}, bits: {} -> (0x{:08X})",
            name, self.value, self.value
        )
    }
}
