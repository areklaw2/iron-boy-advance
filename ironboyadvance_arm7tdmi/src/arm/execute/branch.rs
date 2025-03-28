use crate::{
    CpuAction, CpuState,
    cpu::{Arm7tdmiCpu, LR},
    memory::MemoryInterface,
};

use crate::arm::ArmInstruction;

pub fn execute_branch_and_exchange<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let value = cpu.get_register(instruction.rn() as usize);
    cpu.set_cpu_state(CpuState::from_bits((value & 0x1) as u8));
    cpu.set_pc(value & !0x1);
    cpu.refill_pipeline();
    CpuAction::PipelineFlush
}

pub fn execute_branch_and_branch_with_link<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ArmInstruction,
) -> CpuAction {
    if instruction.link() {
        cpu.set_register(LR, cpu.pc() - 4)
    }
    cpu.set_pc((cpu.pc() as i32).wrapping_add(instruction.offset()) as u32);
    cpu.refill_pipeline();
    CpuAction::PipelineFlush
}
