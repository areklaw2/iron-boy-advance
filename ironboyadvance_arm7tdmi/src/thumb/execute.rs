use crate::{CpuAction, cpu::Arm7tdmiCpu, memory::MemoryInterface};

use super::ThumbInstruction;

pub fn execute_move_shifted_register<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    todo!()
}
