use crate::{
    AluOperationsOpcode, CpuAction, CpuState, Exception, HiRegOpsBxOpcode, MovCmpAddSubImmediateOpcode,
    alu::*,
    barrel_shifter::{ShiftType, asr, lsl, lsr, ror},
    cpu::{Arm7tdmiCpu, LR, PC, SP},
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
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

            cpu.set_negative(result >> 31 != 0);
            cpu.set_zero(result == 0);
            cpu.set_carry(carry);
            result
        }
        LSR => {
            operand2 &= 0xFF;
            let result = lsr(operand1, operand2, &mut carry, false);
            cpu.idle_cycle();
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

            cpu.set_negative(result >> 31 != 0);
            cpu.set_zero(result == 0);
            cpu.set_carry(carry);
            result
        }
        ASR => {
            operand2 &= 0xFF;
            let result = asr(operand1, operand2, &mut carry, false);
            cpu.idle_cycle();
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

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
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

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
            access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

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
    let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
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
                action = CpuAction::PipelineFlush;
            }
        }
        MOV => {
            let result = mov(cpu, false, operand2, cpu.cpsr().carry());
            cpu.set_register(destination, result);
            if destination == PC {
                cpu.set_pc(cpu.pc() & !0x1);
                cpu.pipeline_flush();
                action = CpuAction::PipelineFlush;
            }
        }
        BX => {
            cpu.set_state(CpuState::from_bits((operand2 & 0x1) as u8));
            cpu.set_pc(operand2 & !0x1);
            cpu.pipeline_flush();
            action = CpuAction::PipelineFlush;
        }
    };

    action
}

pub fn execute_pc_relative_load<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ThumbInstruction) -> CpuAction {
    let offset = instruction.offset();
    let address = (cpu.register(PC) & !0x2).wrapping_add((offset << 2) as u32);
    let value = cpu.load_32(address, MemoryAccess::NonSequential as u8);
    cpu.set_register(instruction.rd() as usize, value);
    cpu.idle_cycle();
    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
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
            cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
        }
        (false, true) => {
            let value = cpu.register(rd as usize);
            cpu.store_8(address, value as u8, MemoryAccess::NonSequential as u8);
        }
        (true, false) => {
            let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        (true, true) => {
            let value = cpu.load_8(address, MemoryAccess::NonSequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
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
            cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
        }
        (false, true) => {
            let value = cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        (true, false) => {
            let value = cpu.load_signed_8(address, MemoryAccess::NonSequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        (true, true) => {
            let value = cpu.load_signed_16(address, MemoryAccess::NonSequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
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
            cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
        }
        (false, true) => {
            let value = cpu.register(rd as usize);
            cpu.store_8(address, value as u8, MemoryAccess::NonSequential as u8);
        }
        (true, false) => {
            let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        (true, true) => {
            let value = cpu.load_8(address, MemoryAccess::NonSequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
}

pub fn execute_load_store_halfword<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let immediate = instruction.offset() * 2;
    let rb_value = cpu.register(instruction.rb() as usize);
    let address = rb_value.wrapping_add(immediate as u32);
    let rd = instruction.rd() as usize;
    match instruction.load() {
        true => {
            let value = cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        false => {
            let value = cpu.register(rd as usize);
            cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
}

pub fn execute_sp_relative_load_store<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let immediate = instruction.offset() * 4;
    let sp_value = cpu.register(SP);
    let address = sp_value.wrapping_add(immediate as u32);
    let rd = instruction.rd() as usize;
    match instruction.load() {
        true => {
            let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
            cpu.set_register(rd, value);
            cpu.idle_cycle();
        }
        false => {
            let value = cpu.register(rd as usize);
            cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
}

pub fn execute_load_address<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ThumbInstruction) -> CpuAction {
    let rd = instruction.rd() as usize;
    let offset = instruction.offset() * 4;
    let value = match instruction.sp() {
        true => cpu.register(SP).wrapping_add(offset as u32),
        false => (cpu.pc() & !0b10).wrapping_add(offset as u32),
    };
    cpu.set_register(rd, value);
    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
}

pub fn execute_add_offset_to_sp<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ThumbInstruction) -> CpuAction {
    let offset = instruction.offset() * 4;
    let sp_value = cpu.register(SP);
    let value = match instruction.signed() {
        true => sp_value.wrapping_sub(offset as u32),
        false => sp_value.wrapping_add(offset as u32),
    };
    cpu.set_register(SP, value);
    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
}

pub fn execute_push_pop_registers<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let mut address = cpu.register(SP);
    let register_list = instruction.register_list();
    let store_lr_load_pc = instruction.store_lr_load_pc();

    let mut memory_access = MemoryAccess::NonSequential;
    match instruction.load() {
        true => {
            if register_list.is_empty() && !store_lr_load_pc {
                let value = cpu.load_32(address, memory_access as u8);
                cpu.set_pc(value);
                cpu.set_register(SP, address + 64);
                cpu.pipeline_flush();
                return CpuAction::PipelineFlush;
            }

            for register in register_list.iter() {
                let value = cpu.load_32(address, memory_access as u8);
                cpu.set_register(*register, value);
                memory_access = MemoryAccess::Sequential;
                address += 4
            }

            if store_lr_load_pc {
                let value = cpu.load_32(address, memory_access as u8);
                cpu.set_register(PC, value & !0b1);
                cpu.set_register(SP, address + 4);
                cpu.idle_cycle();
                cpu.pipeline_flush();
                return CpuAction::PipelineFlush;
            }

            cpu.idle_cycle();
            cpu.set_register(SP, address);
        }
        false => {
            if register_list.is_empty() && !store_lr_load_pc {
                address -= 64;
                cpu.set_register(SP, address);
                let value = cpu.pc() + 2;
                cpu.store_32(address, value, memory_access as u8);
                return CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
            }

            address -= register_list.len() as u32 * 4;
            if store_lr_load_pc {
                address -= 4
            }
            cpu.set_register(SP, address);

            for register in register_list.iter() {
                let value = cpu.register(*register);
                cpu.store_32(address, value, memory_access as u8);
                memory_access = MemoryAccess::Sequential;
                address += 4
            }

            if store_lr_load_pc {
                let value = cpu.register(LR);
                cpu.store_32(address, value, memory_access as u8);
            }
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
}

pub fn execute_multiple_load_store<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let rb = instruction.rb() as usize;
    let mut address = cpu.register(rb);
    let register_list = instruction.register_list();

    let mut memory_access = MemoryAccess::NonSequential;
    match instruction.load() {
        true => {
            if register_list.is_empty() {
                let value = cpu.load_32(address, memory_access as u8);
                cpu.set_pc(value);
                cpu.set_register(rb, address + 64);
                cpu.pipeline_flush();
                return CpuAction::PipelineFlush;
            }

            for register in register_list.iter() {
                let value = cpu.load_32(address, memory_access as u8);
                cpu.set_register(*register, value);
                memory_access = MemoryAccess::Sequential;
                address += 4
            }

            cpu.idle_cycle();
            if !register_list.contains(&rb) {
                cpu.set_register(rb, address);
            }
        }
        false => {
            if register_list.is_empty() {
                let value = cpu.pc() + 2;
                cpu.store_32(address, value, memory_access as u8);
                cpu.set_register(rb, address + 64);
                return CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
            }

            for (i, register) in register_list.iter().enumerate() {
                let value = cpu.register(*register);
                cpu.store_32(address, value, memory_access as u8);

                if i == 0 {
                    cpu.set_register(rb, address + register_list.len() as u32 * 4);
                }

                memory_access = MemoryAccess::Sequential;
                address += 4
            }
        }
    }

    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
}

pub fn execute_conditional_branch<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let condition = instruction.cond();
    if !cpu.is_condition_met(condition) {
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    } else {
        let offset = (((instruction.offset() as u32) << 24) as i32) >> 23;
        cpu.set_pc(cpu.pc().wrapping_add(offset as u32));
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }
}

pub fn execute_software_interrupt<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    _instruction: &ThumbInstruction,
) -> CpuAction {
    cpu.exeception(Exception::SoftwareInterrupt);
    CpuAction::PipelineFlush
}

pub fn execute_unconditional_branch<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let offset = (((instruction.offset() as u32) << 21) as i32) >> 20;
    cpu.set_pc(cpu.pc().wrapping_add(offset as u32));
    cpu.pipeline_flush();
    CpuAction::PipelineFlush
}

pub fn execute_long_branch_with_link<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ThumbInstruction,
) -> CpuAction {
    let mut offset = instruction.offset() as i32;
    match instruction.high() {
        true => {
            offset <<= 1;
            let temp = (cpu.pc() - 2) | 0b1;
            cpu.set_pc((cpu.register(LR) & !0b1).wrapping_add(offset as u32));
            cpu.set_register(LR, temp);
            cpu.pipeline_flush();
            CpuAction::PipelineFlush
        }
        false => {
            offset = (offset << 21) >> 9;
            cpu.set_register(LR, cpu.pc().wrapping_add(offset as u32));
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
        }
    }
}
