use crate::Execute;
use ironboyadvance_arm7tdmi::{CpuAction, CpuState, arm::BranchAndExchange, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Execute for BranchAndExchange {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let value = cpu.register(self.rn() as usize);
        cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
        cpu.set_pc(value & !0x1);
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }
}
