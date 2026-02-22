use crate::{
    BitOps, CpuAction, CpuState, Register,
    arm::arm_instruction,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};

#[derive(Debug, Clone, Copy)]
pub struct BranchAndExchange {
    value: u32,
}

arm_instruction!(BranchAndExchange);

impl Instruction for BranchAndExchange {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let value = cpu.register(self.rn() as usize);
        cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
        cpu.set_pc(value & !0x1);
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let rn = self.rn();
        format!("BX{cond} {rn}")
    }
}

impl BranchAndExchange {
    #[inline]
    fn rn(&self) -> Register {
        self.value.bits(0..=3).into()
    }
}
