use getset::CopyGetters;

use crate::{
    BitOps, Condition, CpuAction, CpuState, Register,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};

#[derive(Debug, Clone, Copy, CopyGetters)]
pub struct BranchAndExchange {
    #[getset(get_copy = "pub(crate)")]
    cond: Condition,
    rn: Register,
}

impl BranchAndExchange {
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(0..=3).into(),
        }
    }
}

impl Instruction for BranchAndExchange {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let value = cpu.register(self.rn as usize);
        cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
        cpu.set_pc(value & !0x1);
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let rn = self.rn;
        format!("BX{cond} {rn}")
    }
}
