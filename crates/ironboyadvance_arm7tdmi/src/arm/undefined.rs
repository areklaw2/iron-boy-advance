use crate::{CpuAction, Exception, arm::arm_instruction, cpu::Arm7tdmiCpu, memory::MemoryInterface};

#[derive(Debug, Clone, Copy, Default)]
pub struct Undefined {
    value: u32,
}

arm_instruction!(Undefined);

impl Undefined {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::Undefined);
        CpuAction::PipelineFlush
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        "Undefined".into()
    }
}
