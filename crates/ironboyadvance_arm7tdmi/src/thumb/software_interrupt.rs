use crate::{
    BitOps, CpuAction, Exception,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};

#[derive(Debug, Clone, Copy)]
pub struct SoftwareInterrupt {
    offset: u16,
}

impl SoftwareInterrupt {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            offset: value.bits(0..=7),
        }
    }
}

impl Instruction for SoftwareInterrupt {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::SoftwareInterrupt);
        CpuAction::PipelineFlush
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        format!("SWI #{}", offset)
    }
}
