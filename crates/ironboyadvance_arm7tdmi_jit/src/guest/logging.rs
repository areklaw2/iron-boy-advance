use ironboyadvance_arm7tdmi::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface};
use tracing::debug;

use crate::{decode_arm, decode_thumb};

pub unsafe extern "C" fn log_instruction<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, encoding: u32, arm: u32) {
    let cpu = unsafe { &mut *cpu };
    match arm != 0 {
        true => debug!("{}", decode_arm(encoding).disassemble(cpu)),
        false => debug!("{}", decode_thumb(encoding as u16).disassemble(cpu)),
    }
}
