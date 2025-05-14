use crate::{
    AluOperationsOpcode, CpuAction, CpuState, HiRegOpsBxOpcode, MovCmpAddSubImmediateOpcode,
    alu::*,
    barrel_shifter::{ShiftType, asr, lsl, lsr, ror},
    cpu::{Arm7tdmiCpu, PC},
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

    let result = match instruction.opcode() != 0 {
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
    use MovCmpAddSubImmediateOpcode::*;
    let rd = instruction.rd() as usize;
    let operand1 = cpu.register(rd);
    let offset = instruction.offset();
    let opcode: MovCmpAddSubImmediateOpcode = instruction.opcode().into();
    let result = match opcode {
        MOV => mov(cpu, true, offset as u32, cpu.cpsr().carry()),
        CMP => cmp(cpu, true, operand1, offset as u32),
        ADD => add(cpu, true, operand1, offset as u32),
        SUB => sub(cpu, true, operand1, offset as u32),
    };

    if opcode != CMP {
        cpu.set_register(rd, result);
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
}

pub fn execute_alu_operations<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ThumbInstruction) -> CpuAction {
    use AluOperationsOpcode::*;
    let rd = instruction.rd() as usize;
    let operand1 = cpu.register(rd);
    let mut operand2 = cpu.register(instruction.rs() as usize);
    let mut carry = cpu.cpsr().carry();
    let opcode: AluOperationsOpcode = instruction.opcode().into();
    let mut access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);

    let result = match opcode {
        AND => and(cpu, true, operand1, operand2, carry),
        EOR => eor(cpu, true, operand1, operand2, carry),
        LSL => {
            operand2 &= 0xFF;
            let result = lsl(operand1, operand2, &mut carry);
            cpu.idle_cycle();
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential);

            cpu.set_negative(result >> 31 != 0);
            cpu.set_zero(result == 0);
            cpu.set_carry(carry);
            result
        }
        LSR => {
            operand2 &= 0xFF;
            let result = lsr(operand1, operand2, &mut carry, false);
            cpu.idle_cycle();
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential);

            cpu.set_negative(result >> 31 != 0);
            cpu.set_zero(result == 0);
            cpu.set_carry(carry);
            result
        }
        ASR => {
            operand2 &= 0xFF;
            let result = asr(operand1, operand2, &mut carry, false);
            cpu.idle_cycle();
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential);

            cpu.set_negative(result >> 31 != 0);
            cpu.set_zero(result == 0);
            cpu.set_carry(carry);
            result
        }
        ADC => adc(cpu, true, operand1, operand2),
        SBC => sbc(cpu, true, operand1, operand2),
        ROR => {
            operand2 &= 0xFF;
            let result = ror(operand1, operand2, &mut carry, false);
            cpu.idle_cycle();
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential);

            cpu.set_negative(result >> 31 != 0);
            cpu.set_zero(result == 0);
            cpu.set_carry(carry);
            result
        }
        TST => tst(cpu, true, operand1, operand2, carry),
        NEG => sub(cpu, true, 0, operand2),
        CMP => cmp(cpu, true, operand1, operand2),
        CMN => cmn(cpu, true, operand1, operand2),
        ORR => orr(cpu, true, operand1, operand2, carry),
        MUL => {
            let multiplier_cycles = multiplier_array_cycles(operand1);
            for _ in 0..multiplier_cycles {
                cpu.idle_cycle();
            }
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential);

            let result = operand1.wrapping_mul(operand2);
            cpu.set_negative(result >> 31 != 0);
            cpu.set_zero(result == 0);
            result
        }
        BIC => bic(cpu, true, operand1, operand2, carry),
        MVN => mvn(cpu, true, operand2, carry),
    };

    if ![TST, CMP, CMN].contains(&opcode) {
        cpu.set_register(rd, result);
    }

    access
}

pub fn execute_hi_register_operations_branch_exchange<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    use HiRegOpsBxOpcode::*;
    let mut access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
    let destination = match instruction.h1() {
        true => instruction.hd() as usize + 8,
        false => instruction.rd() as usize,
    };
    let operand1 = cpu.register(destination);

    let source = match instruction.h2() {
        true => instruction.hs() as usize + 8,
        false => instruction.rs() as usize,
    };
    let mut operand2 = cpu.register(source);
    if source == PC {
        operand2 &= !0x1
    }

    match instruction.opcode().into() {
        CMP => {
            cmp(cpu, true, operand1, operand2);
        }
        ADD => {
            let result = add(cpu, false, operand1, operand2);
            cpu.set_register(destination, result);
            if destination == PC {
                cpu.set_pc(cpu.pc() & !0x1);
                cpu.pipeline_flush();
                access = CpuAction::PipelineFlush;
            }
        }
        MOV => {
            let result = mov(cpu, false, operand2, cpu.cpsr().carry());
            cpu.set_register(destination, result);
            if destination == PC {
                cpu.set_pc(cpu.pc() & !0x1);
                cpu.pipeline_flush();
                access = CpuAction::PipelineFlush;
            }
        }
        BX => {
            cpu.set_state(CpuState::from_bits((operand2 & 0x1) as u8));
            cpu.set_pc(operand2 & !0x1);
            cpu.pipeline_flush();
            access = CpuAction::PipelineFlush;
        }
    };

    access
}

pub fn execute_pc_relative_load<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ThumbInstruction) -> CpuAction {
    let offset = instruction.offset();
    let address = (cpu.register(PC) & !0x2).wrapping_add((offset << 2) as u32);
    let value = cpu.load_32(address, MemoryAccess::Nonsequential as u8);
    cpu.set_register(instruction.rd() as usize, value);
    cpu.idle_cycle();
    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential)
}

pub fn execute_load_store_register_offset<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let ro_value = cpu.register(instruction.ro() as usize);
    let rb_value = cpu.register(instruction.rb() as usize);
    let address = rb_value.wrapping_add(ro_value);

    let rd = instruction.rd() as usize;
    let byte = instruction.byte();
    let load = instruction.load();
    match (load, byte) {
        (false, false) => {
            let value = cpu.register(rd as usize);
            cpu.store_32(address, value, MemoryAccess::Nonsequential as u8);
        }
        (false, true) => {
            let value = cpu.register(rd as usize);
            cpu.store_8(address, value as u8, MemoryAccess::Nonsequential as u8);
        }
        (true, false) => {
            let value = cpu.load_rotated_32(address, MemoryAccess::Nonsequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        (true, true) => {
            let value = cpu.load_8(address, MemoryAccess::Nonsequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential)
}

pub fn execute_load_store_sign_extended_byte_halfword<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let ro_value = cpu.register(instruction.ro() as usize);
    let rb_value = cpu.register(instruction.rb() as usize);
    let address = rb_value.wrapping_add(ro_value);

    let rd = instruction.rd() as usize;
    let signed = instruction.signed();
    let halfword = instruction.halfword();
    match (signed, halfword) {
        (false, false) => {
            let value = cpu.register(rd as usize);
            cpu.store_16(address, value as u16, MemoryAccess::Nonsequential as u8);
        }
        (false, true) => {
            let value = cpu.load_rotated_16(address, MemoryAccess::Nonsequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        (true, false) => {
            let value = cpu.load_signed_8(address, MemoryAccess::Nonsequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        (true, true) => {
            let value = cpu.load_signed_16(address, MemoryAccess::Nonsequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential)
}

pub fn execute_load_store_immediate_offset<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let immediate = instruction.offset() * 4;
    let rb_value = cpu.register(instruction.rb() as usize);
    let address = rb_value.wrapping_add(immediate as u32);

    let rd = instruction.rd() as usize;
    let byte = instruction.byte();
    let load = instruction.load();
    match (load, byte) {
        (false, false) => {
            let value = cpu.register(rd as usize);
            cpu.store_32(address, value, MemoryAccess::Nonsequential as u8);
        }
        (false, true) => {
            let value = cpu.register(rd as usize);
            cpu.store_8(address, value as u8, MemoryAccess::Nonsequential as u8);
        }
        (true, false) => {
            let value = cpu.load_rotated_32(address, MemoryAccess::Nonsequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        (true, true) => {
            let value = cpu.load_8(address, MemoryAccess::Nonsequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential)
}
