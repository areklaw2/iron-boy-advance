use ironboyadvance_utils::bit::BitOps;

use crate::{Condition, CpuAction, CpuState, Register, cpu::Arm7tdmiCpu, memory::MemoryInterface};

#[derive(Debug, Clone, Copy)]
pub struct BranchAndExchange {
    value: u32,
}

impl BranchAndExchange {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let value = cpu.register(self.rn() as usize);
        cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
        cpu.set_pc(value & !0x1);
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }

    pub fn disassemble(&self) -> String {
        let cond = self.cond();
        let rn = self.rn();
        format!("BX{cond} {rn}")
    }

    #[inline]
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }

    #[inline]
    fn rn(&self) -> Register {
        self.value.bits(0..=3).into()
    }
}
