use crate::{
    CpuAction,
    alu::{MovCmpAddSubImmdiateOpcode, add, cmp, mov, sub},
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
    let offset5 = instruction.offset() as u32;

    let value = cpu.register(rs);
    let mut carry = cpu.cpsr().carry();
    let result = match instruction.opcode().into() {
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

pub fn execute_add_subtract<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ThumbInstruction) -> CpuAction {
    let rd = instruction.rd() as usize;
    let operand1 = cpu.register(instruction.rs() as usize);
    let operand2 = match instruction.is_immediate() {
        true => instruction.offset() as u32,
        false => cpu.register(instruction.rn() as usize),
    };

    let subtract = instruction.opcode() != 0;
    let result = match subtract {
        true => sub(cpu, true, operand1, operand2),
        false => add(cpu, true, operand1, operand2),
    };

    cpu.set_register(rd, result);
    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
}

pub fn execute_move_compare_add_subtract_immediate<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    use MovCmpAddSubImmdiateOpcode::*;
    let rd = instruction.rd() as usize;
    let operand1 = cpu.register(rd);
    let offset = instruction.offset();
    match instruction.opcode().into() {
        MOV => {
            let result = mov(cpu, true, offset as u32, cpu.cpsr().carry());
            cpu.set_register(rd, result);
        }
        CMP => {
            cmp(cpu, true, operand1, offset as u32);
        }
        ADD => {
            let result = add(cpu, true, operand1, offset as u32);
            cpu.set_register(rd, result);
        }
        SUB => {
            let result = sub(cpu, true, operand1, offset as u32);
            cpu.set_register(rd, result);
        }
    };

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
}
