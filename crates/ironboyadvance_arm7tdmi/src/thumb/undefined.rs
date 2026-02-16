use crate::{CpuAction, Exception, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::thumb_instruction};

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub struct Undefined {
    value: u16,
}

thumb_instruction!(Undefined);

impl Undefined {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::Undefined);
        CpuAction::PipelineFlush
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        "Undefined".into()
    }
}
