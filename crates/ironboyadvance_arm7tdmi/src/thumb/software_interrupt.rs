use crate::{
    BitOps, CpuAction, Exception,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct SoftwareInterrupt {
    value: u16,
}

thumb_instruction!(SoftwareInterrupt);

impl Instruction for SoftwareInterrupt {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::SoftwareInterrupt);
        CpuAction::PipelineFlush
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        format!("SWI #{}", offset)
    }
}

impl SoftwareInterrupt {
    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(0..=7)
    }
}
