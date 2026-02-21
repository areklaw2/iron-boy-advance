use bitfields::bitfield;

use crate::{Condition, CpuAction, CpuState, Register, cpu::Arm7tdmiCpu, memory::MemoryInterface};

#[bitfield(u32)]
#[derive(Clone, Copy)]
pub struct BranchAndExchange {
    #[bits(4)]
    rn: Register,
    #[bits(24)]
    _reserved: u32,
    #[bits(4)]
    cond: Condition,
}

impl BranchAndExchange {
    pub(crate) fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let value = cpu.register(self.rn() as usize);
        cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
        cpu.set_pc(value & !0x1);
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let rn = self.rn();
        format!("BX{cond} {rn}")
    }
}
