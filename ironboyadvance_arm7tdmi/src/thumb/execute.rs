use crate::{
    CpuAction,
    barrel_shifter::{ShiftType, asr, lsl, lsr},
    cpu::Arm7tdmiCpu,
    memory::{MemoryAccess, MemoryInterface},
};

use super::ThumbInstruction;

pub fn execute_move_shifted_register<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let rd = instruction.rd() as usize;
    let rs = instruction.rs() as usize;
    let offset5 = instruction.offset5() as u32;

    let value = cpu.register(rs);
    let mut carry = cpu.cpsr().carry();
    let result = match instruction.opcode() {
        ShiftType::LSL => lsl(value, offset5, &mut carry),
        ShiftType::LSR => lsr(value, offset5, &mut carry, true),
        ShiftType::ASR => asr(value, offset5, &mut carry, true),
        ShiftType::ROR => unimplemented!(),
    };

    cpu.set_negative(result >> 31 != 0);
    cpu.set_zero(result == 0);
    cpu.set_carry(carry);

    cpu.set_register(rd, result);
    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
}
