use crate::Execute;
use ironboyadvance_arm7tdmi::{CpuAction, Exception, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::Undefined};

impl Execute for Undefined {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::Undefined);
        CpuAction::PipelineFlush
    }
}
