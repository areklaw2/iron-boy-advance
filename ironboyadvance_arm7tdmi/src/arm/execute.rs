use bitvec::field::BitField;

use crate::{
    CpuAction, CpuMode, CpuState, DataProcessingOpcode, Exception,
    alu::*,
    barrel_shifter::{ShiftBy, ShiftType, asr, lsl, lsr, ror},
    cpu::{Arm7tdmiCpu, LR, PC},
    memory::{MemoryAccess, MemoryInterface},
    psr::ProgramStatusRegister,
};
use DataProcessingOpcode::*;

use crate::arm::ArmInstruction;

pub fn execute_branch_exchange<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let value = cpu.register(instruction.rn() as usize);
    cpu.set_state(CpuState::from_bits((value & 0x1) as u8));
    cpu.set_pc(value & !0x1);
    cpu.pipeline_flush();
    CpuAction::PipelineFlush
}

pub fn execute_branch_and_branch_link<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ArmInstruction,
) -> CpuAction {
    if instruction.link() {
        cpu.set_register(LR, cpu.pc() - 4)
    }

    let offset = (((instruction.offset() as u32) << 8) as i32) >> 6;
    cpu.set_pc((cpu.pc() as i32).wrapping_add(offset) as u32);
    cpu.pipeline_flush();
    CpuAction::PipelineFlush
}

pub fn execute_data_processing<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let mut cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
    let rn = instruction.rn() as usize;
    let mut operand1 = cpu.register(rn);
    let mut carry = cpu.cpsr().carry();

    let operand2 = match instruction.is_immediate() {
        true => {
            let rotate = 2 * instruction.rotate();
            let immediate = instruction.immediate();
            ror(immediate, rotate, &mut carry, false)
        }
        false => {
            let rm = instruction.rm() as usize;
            let mut rm_value = cpu.register(rm);
            let shift_by = instruction.shift_by();
            let shift_amount = match shift_by {
                ShiftBy::Immediate => instruction.shift_amount(),
                ShiftBy::Register => {
                    if rn == PC {
                        operand1 += 4;
                    }
                    if rm == PC {
                        rm_value += 4;
                    }
                    cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential);
                    cpu.idle_cycle();
                    cpu.register(instruction.rs() as usize) & 0xFF
                }
            };
            match instruction.shift_type() {
                ShiftType::LSL => lsl(rm_value, shift_amount, &mut carry),
                ShiftType::LSR => lsr(rm_value, shift_amount, &mut carry, shift_by.into()),
                ShiftType::ASR => asr(rm_value, shift_amount, &mut carry, shift_by.into()),
                ShiftType::ROR => ror(rm_value, shift_amount, &mut carry, shift_by.into()),
            }
        }
    };

    let set_flags = instruction.sets_flags();
    let opcode = instruction.opcode();
    let result = match opcode {
        AND => and(cpu, set_flags, operand1, operand2, carry),
        EOR => eor(cpu, set_flags, operand1, operand2, carry),
        SUB => sub(cpu, set_flags, operand1, operand2),
        RSB => rsb(cpu, set_flags, operand2, operand1),
        ADD => add(cpu, set_flags, operand1, operand2),
        ADC => adc(cpu, set_flags, operand1, operand2),
        SBC => sbc(cpu, set_flags, operand1, operand2),
        RSC => rsc(cpu, set_flags, operand2, operand1),
        TST => tst(cpu, set_flags, operand1, operand2, carry),
        TEQ => teq(cpu, set_flags, operand1, operand2, carry),
        CMP => cmp(cpu, set_flags, operand1, operand2),
        CMN => cmn(cpu, set_flags, operand1, operand2),
        ORR => orr(cpu, set_flags, operand1, operand2, carry),
        MOV => mov(cpu, set_flags, operand2, carry),
        BIC => bic(cpu, set_flags, operand1, operand2, carry),
        MVN => mvn(cpu, set_flags, operand2, carry),
    };

    let rd = instruction.rd() as usize;
    if set_flags && rd == PC {
        let spsr = cpu.spsr();
        cpu.set_cpsr(spsr);
    }

    if !matches!(opcode, TST | TEQ | CMP | CMN) {
        cpu.set_register(rd, result);
        if rd == PC {
            cpu.pipeline_flush();
            cpu_action = CpuAction::PipelineFlush
        }
    }

    cpu_action
}

pub fn execute_psr_transfer<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let is_spsr = instruction.is_spsr();
    match instruction.bits[16..=21].load::<u8>() == 0xF {
        true => {
            let rd = instruction.rd() as usize;
            let psr = match is_spsr {
                false => cpu.cpsr(),
                true => cpu.spsr(),
            };
            cpu.set_register(rd, psr.into_bits());
        }
        false => {
            let mut mask = 0u32;
            if instruction.bits[19] {
                mask |= 0xFF000000;
            }
            if instruction.bits[18] {
                mask |= 0xFF0000;
            }
            if instruction.bits[17] {
                mask |= 0xFF00;
            }
            if instruction.bits[16] {
                mask |= 0xFF;
            }

            let mut operand = match instruction.is_immediate() {
                false => cpu.register(instruction.rm() as usize),
                true => {
                    let mut carry = cpu.cpsr().carry();
                    let rotate = 2 * instruction.rotate();
                    let immediate = instruction.immediate();
                    ror(immediate, rotate, &mut carry, false)
                }
            };

            match is_spsr {
                false => {
                    // User mode can only change flags
                    if cpu.cpsr().mode() == CpuMode::User {
                        mask &= 0xFF000000;
                    }

                    // Make sure operand has the 4th bit
                    if mask & 0xFF != 0 {
                        operand |= 0x10;
                    }

                    let bits = (cpu.cpsr().into_bits() & !mask) | (operand & mask);
                    cpu.set_cpsr(ProgramStatusRegister::from_bits(bits));
                }
                true => {
                    if cpu.cpsr().mode() != CpuMode::User && cpu.cpsr().mode() != CpuMode::System {
                        let bits = (cpu.spsr().into_bits() & !mask) | (operand & mask);
                        cpu.set_spsr(ProgramStatusRegister::from_bits(bits));
                    }
                }
            }
        }
    }
    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
}

pub fn execute_multiply<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let rd = instruction.rd() as usize;
    let rm = instruction.rm() as usize;
    let rs = instruction.rs() as usize;
    let rn = instruction.rn() as usize;

    let mut operand1 = cpu.register(rm);
    if rm == PC {
        operand1 += 4
    }
    let mut operand2 = cpu.register(rs);
    if rs == PC {
        operand2 += 4
    }

    let mut result = operand1.wrapping_mul(operand2);
    let multiplier_cycles = multiplier_array_cycles(operand2);
    for _ in 0..multiplier_cycles {
        cpu.idle_cycle();
    }

    if instruction.accumulate() {
        let mut accumulator = cpu.register(rn);
        if rn == PC {
            accumulator += 4
        }
        result = result.wrapping_add(accumulator);
        cpu.idle_cycle();
    };

    if instruction.sets_flags() {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
    }

    cpu.set_register(rd, result);
    match rd == PC {
        true => {
            cpu.pipeline_flush();
            CpuAction::PipelineFlush
        }
        false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential),
    }
}

pub fn execute_multiply_long<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let rd_lo = instruction.rd_lo() as usize;
    let rd_hi = instruction.rd_hi() as usize;
    let rm = instruction.rm() as usize;
    let rs = instruction.rs() as usize;

    let mut operand1 = cpu.register(rm);
    if rm == PC {
        operand1 += 4
    }
    let mut operand2 = cpu.register(rs);
    if rs == PC {
        operand2 += 4
    }

    let unsigned = instruction.unsigned();
    let mut result = match unsigned {
        true => (operand1 as i32 as i64).wrapping_mul(operand2 as i32 as i64) as u64,
        false => (operand1 as u64).wrapping_mul(operand2 as u64),
    };

    let multiplier_cycles = multiplier_array_cycles(operand2);
    for _ in 0..multiplier_cycles {
        cpu.idle_cycle();
    }

    if instruction.accumulate() {
        let mut accumulator_lo = cpu.register(rd_lo) as u64;
        if rd_lo == PC {
            accumulator_lo += 4
        }
        let mut accumulator_hi = cpu.register(rd_hi) as u64;
        if rd_hi == PC {
            accumulator_hi += 4
        }
        result = result.wrapping_add(accumulator_hi << 32 | accumulator_lo);
        cpu.idle_cycle();
    };

    let result_lo = (result & 0xFFFFFFFF) as u32;
    let result_hi = (result >> 32) as u32;
    if instruction.sets_flags() {
        cpu.set_negative(result_hi >> 31 != 0);
        cpu.set_zero(result == 0);
    }

    cpu.set_register(rd_lo, result_lo);
    cpu.set_register(rd_hi, result_hi);
    match rd_hi == PC || rd_lo == PC {
        true => {
            cpu.pipeline_flush();
            CpuAction::PipelineFlush
        }
        false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential),
    }
}

pub fn execute_single_data_transfer<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ArmInstruction,
) -> CpuAction {
    let rd = instruction.rd() as usize;
    let rn = instruction.rn() as usize;

    let mut address = cpu.register(rn);
    let mut offset = match instruction.is_immediate() {
        true => instruction.immediate(),
        false => {
            let rm_value = cpu.register(instruction.rm() as usize);
            let shift_amount = instruction.shift_amount();
            let mut carry = cpu.cpsr().carry();
            match instruction.shift_type() {
                ShiftType::LSL => lsl(rm_value, shift_amount, &mut carry),
                ShiftType::LSR => lsr(rm_value, shift_amount, &mut carry, true),
                ShiftType::ASR => asr(rm_value, shift_amount, &mut carry, true),
                ShiftType::ROR => ror(rm_value, shift_amount, &mut carry, true),
            }
        }
    };

    if !instruction.add() {
        offset = (-(offset as i64)) as u32
    }

    let pre_index = instruction.pre_index();
    if pre_index {
        address = address.wrapping_add(offset)
    }

    let load = instruction.load();
    let byte = instruction.byte();
    let write_back = instruction.write_back();
    match load {
        true => {
            let value = match byte {
                true => cpu.load_8(address, MemoryAccess::Nonsequential as u8),
                false => cpu.load_rotated_32(address, MemoryAccess::Nonsequential as u8),
            };
            if write_back || !pre_index {
                if rn != rd && rn == PC {
                    cpu.pipeline_flush();
                }
                cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
            }
            cpu.idle_cycle();
            cpu.set_register(rd, value);
        }
        false => {
            let mut value = cpu.register(rd);
            if rd == PC {
                value += 4;
            }
            match byte {
                true => cpu.store_8(address, value as u8, MemoryAccess::Nonsequential as u8),
                false => cpu.store_32(address, value, MemoryAccess::Nonsequential as u8),
            };
            if write_back || !pre_index {
                if rn == PC {
                    cpu.pipeline_flush();
                }
                cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
            }
        }
    }

    match load && rd == PC {
        true => {
            cpu.pipeline_flush();
            CpuAction::PipelineFlush
        }
        false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential),
    }
}

pub fn execute_halfword_and_signed_data_transfer<I: MemoryInterface>(
    cpu: &mut Arm7tdmiCpu<I>,
    instruction: &ArmInstruction,
) -> CpuAction {
    let rd = instruction.rd() as usize;
    let rn = instruction.rn() as usize;

    let mut address = cpu.register(rn);
    let mut offset = match instruction.is_immediate() {
        true => instruction.immediate_hi() << 4 | instruction.immediate_lo(),
        false => cpu.register(instruction.rm() as usize),
    };

    if !instruction.add() {
        offset = (-(offset as i64)) as u32
    }

    let pre_index = instruction.pre_index();
    if pre_index {
        address = address.wrapping_add(offset)
    }

    let load = instruction.load();
    let write_back = instruction.write_back();
    let s = instruction.signed();
    let h = instruction.halfword();
    match load {
        true => match (s, h) {
            (false, false) => {}
            (false, true) => {
                let value = cpu.load_rotated_16(address, MemoryAccess::Nonsequential as u8);
                if write_back || !pre_index {
                    if rn != rd && rn == PC {
                        cpu.pipeline_flush();
                    }
                    cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                }
                cpu.idle_cycle();
                cpu.set_register(rd, value);
            }
            (true, false) => {
                let value = cpu.load_signed_8(address, MemoryAccess::Nonsequential as u8);
                if write_back || !pre_index {
                    if rn != rd && rn == PC {
                        cpu.pipeline_flush();
                    }
                    cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                }
                cpu.idle_cycle();
                cpu.set_register(rd, value);
            }
            (true, true) => {
                let value = cpu.load_signed_16(address, MemoryAccess::Nonsequential as u8);
                if write_back || !pre_index {
                    if rn != rd && rn == PC {
                        cpu.pipeline_flush();
                    }
                    cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                }
                cpu.idle_cycle();
                cpu.set_register(rd, value);
            }
        },
        false => {
            let mut value = cpu.register(rd);
            if rd == PC {
                value += 4;
            }
            match (s, h) {
                (false, false) => {}
                (false, true) => {
                    cpu.store_16(address, value as u16, MemoryAccess::Nonsequential as u8);
                    if write_back || !pre_index {
                        if rn == PC {
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                    }
                }
                (true, false) => {
                    cpu.idle_cycle();
                    if write_back || !pre_index {
                        if rn == PC {
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                    }
                }
                (true, true) => {
                    cpu.idle_cycle();
                    if write_back || !pre_index {
                        if rn == PC {
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                    }
                }
            };
        }
    }

    match load && rd == PC {
        true => {
            cpu.pipeline_flush();
            CpuAction::PipelineFlush
        }
        false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential),
    }
}

pub fn execute_block_data_transfer<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let mut register_list = instruction.register_list();
    let rn = instruction.rn() as usize;
    let mut address = cpu.register(rn);

    let mut transfer_pc = register_list.contains(&PC);
    let transfer_bytes = if !register_list.is_empty() {
        register_list.len() as u32 * 4
    } else {
        register_list.push(PC);
        transfer_pc = true;
        64
    };

    let load = instruction.load();
    let load_psr_force_user = instruction.load_psr_force_user();
    let mode = cpu.cpsr().mode();
    let switch_mode = load_psr_force_user && (!load || !transfer_pc) && ![CpuMode::User, CpuMode::System].contains(&mode);
    if switch_mode {
        cpu.set_mode(CpuMode::User);
    }

    let add = instruction.add();
    let mut pre_index = instruction.pre_index();
    let mut base_address = address;
    if !add {
        pre_index = !pre_index;
        address -= transfer_bytes;
        base_address -= transfer_bytes;
    } else {
        base_address += transfer_bytes
    }

    let write_back = instruction.write_back();
    let mut memory_access = MemoryAccess::Nonsequential;
    let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential);
    match load {
        true => {
            for (i, register) in register_list.iter().enumerate() {
                if pre_index {
                    address += 4
                }

                let value = cpu.load_32(address, memory_access as u8);
                if write_back && i == 0 {
                    if rn == PC {
                        base_address += 4;
                        if !transfer_pc {
                            cpu.pipeline_flush();
                        }
                    }
                    cpu.set_register(rn, base_address);
                }
                cpu.set_register(*register, value);

                if !pre_index {
                    address += 4
                }

                memory_access = MemoryAccess::Sequential;
            }

            cpu.idle_cycle();
            if transfer_pc {
                if load_psr_force_user {
                    cpu.set_cpsr(cpu.spsr());
                }

                cpu.pipeline_flush();
                action = CpuAction::PipelineFlush;
            }
        }
        false => {
            for (i, register) in register_list.iter().enumerate() {
                if pre_index {
                    address += 4
                }

                let mut value = cpu.register(*register);
                if *register == PC {
                    match write_back && rn == PC {
                        true => value -= 4,
                        false => value += 4,
                    }
                }

                cpu.store_32(address, value, memory_access as u8);
                if write_back && i == 0 {
                    if rn == PC {
                        base_address += 4;
                        cpu.pipeline_flush();
                    }
                    cpu.set_register(rn, base_address);
                }

                if !pre_index {
                    address += 4
                }

                memory_access = MemoryAccess::Sequential;
            }
        }
    }

    if switch_mode {
        cpu.set_mode(mode);
    }

    action
}

pub fn execute_single_data_swap<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let rd = instruction.rd() as usize;
    let rn = instruction.rn() as usize;
    let rm = instruction.rm() as usize;

    let address = cpu.register(rn);
    let mut source = cpu.register(rm);
    if rm == PC {
        source += 4;
    }

    let value: u32;
    match instruction.byte() {
        true => {
            value = cpu.load_8(address, MemoryAccess::Nonsequential as u8);
            cpu.store_8(address, source as u8, MemoryAccess::Nonsequential | MemoryAccess::Lock);
        }
        false => {
            value = cpu.load_rotated_32(address, MemoryAccess::Nonsequential as u8);
            cpu.store_32(address, source, MemoryAccess::Nonsequential | MemoryAccess::Lock);
        }
    };

    cpu.idle_cycle();
    cpu.set_register(rd, value);
    match rd == PC {
        true => {
            cpu.pipeline_flush();
            CpuAction::PipelineFlush
        }
        false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential),
    }
}

pub fn execute_software_interrupt<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, _instruction: &ArmInstruction) -> CpuAction {
    cpu.exeception(Exception::SoftwareInterrupt);
    CpuAction::PipelineFlush
}

pub fn execute_undefined<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, _instruction: &ArmInstruction) -> CpuAction {
    cpu.exeception(Exception::Undefined);
    CpuAction::PipelineFlush
}
