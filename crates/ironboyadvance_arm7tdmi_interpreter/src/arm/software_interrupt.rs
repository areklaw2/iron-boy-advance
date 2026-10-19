use crate::Execute;
use ironboyadvance_arm7tdmi::{CpuAction, Exception, arm::SoftwareInterrupt, cpu::Arm7tdmiCpu, memory::MemoryInterface};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for SoftwareInterrupt {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match !cpu.bios_loaded() && cpu.bios_call(self.comment() >> 16) {
            true => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
            false => {
                cpu.exception(Exception::SoftwareInterrupt);
                CpuAction::PipelineFlush
            }
        }
    }
}
