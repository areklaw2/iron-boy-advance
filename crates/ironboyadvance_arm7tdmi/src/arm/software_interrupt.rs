use crate::{
    BitOps, CpuAction, Exception,
    arm::arm_instruction,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};

#[derive(Debug, Clone, Copy)]
pub struct SoftwareInterrupt {
    value: u32,
}

arm_instruction!(SoftwareInterrupt);

impl Instruction for SoftwareInterrupt {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::SoftwareInterrupt);
        CpuAction::PipelineFlush
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let comment = self.comment();
        format!("SWI{} 0x{:08X}", cond, comment)
    }
}

impl SoftwareInterrupt {
    #[inline]
    pub fn comment(&self) -> u32 {
        self.value.bits(0..=23)
    }
}
