use bitvec::field::BitField;

use crate::{
    CpuAction, CpuMode, CpuState, Register,
    alu::{AluInstruction::*, *},
    barrel_shifter::{ShiftBy, ShiftType, asr, lsl, lsr, ror},
    cpu::{Arm7tdmiCpu, LR, PC},
    memory::{MemoryAccess, MemoryInterface},
    psr::ProgramStatusRegister,
};

use crate::arm::ArmInstruction;

pub fn execute_bx<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let value = cpu.register(instruction.rn() as usize);
    cpu.set_state(CpuState::from_bits((value & 0x1) as u8));
    cpu.set_pc(value & !0x1);
    cpu.pipeline_flush();
    CpuAction::PipelineFlush
}

pub fn execute_b_bl<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    if instruction.link() {
        cpu.set_register(LR, cpu.pc() - 4)
    }
    cpu.set_pc((cpu.pc() as i32).wrapping_add(instruction.offset()) as u32);
    cpu.pipeline_flush();
    CpuAction::PipelineFlush
}

pub fn execute_data_processing<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let mut cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
    let rn = instruction.rn() as usize;
    let mut operand1 = cpu.register(rn);
    let mut carry = cpu.cpsr().carry();

    let operand2 = match instruction.is_immediate_operand() {
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
        RSB => sub(cpu, set_flags, operand2, operand1),
        ADD => add(cpu, set_flags, operand1, operand2),
        ADC => adc(cpu, set_flags, operand1, operand2),
        SBC => sbc(cpu, set_flags, operand1, operand2),
        RSC => sbc(cpu, set_flags, operand2, operand1),
        TST => and(cpu, set_flags, operand1, operand2, carry),
        TEQ => eor(cpu, set_flags, operand1, operand2, carry),
        CMP => sub(cpu, set_flags, operand1, operand2),
        CMN => add(cpu, set_flags, operand1, operand2),
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
    let cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
    let is_spsr = instruction.is_spsr();

    //MRS
    if instruction.bits[16..=21].load::<u8>() == 0xF {
        let rd = instruction.rd() as usize;
        let psr = match is_spsr {
            false => cpu.cpsr(),
            true => cpu.spsr(),
        };
        cpu.set_register(rd, psr.into_bits());
        return cpu_action;
    }

    //MSR
    let mut operand = match instruction.is_immediate_operand() {
        false => cpu.register(instruction.rm() as usize),
        true => {
            let mut carry = cpu.cpsr().carry();
            let rotate = 2 * instruction.rotate();
            let immediate = instruction.immediate();
            let value = ror(immediate, rotate, &mut carry, false);
            cpu.set_carry(carry);
            value
        }
    };

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

    match is_spsr {
        true => {
            debug_assert!(cpu.cpsr().mode() != CpuMode::User && cpu.cpsr().mode() != CpuMode::System);
            let new_psr = ProgramStatusRegister::from_bits((cpu.spsr().into_bits() & !mask) | (operand & mask));
            cpu.set_spsr(new_psr);
        }
        false => {
            if cpu.cpsr().mode() != CpuMode::User && cpu.cpsr().mode() != CpuMode::System {
                mask &= 0xFF000000;
                println!("flag");
                println!("{:032b}", (cpu.cpsr().into_bits() & !mask) | (operand & mask))
            }

            let new_psr = ProgramStatusRegister::from_bits((cpu.cpsr().into_bits() & !mask) | (operand & mask));
            cpu.set_cpsr(new_psr);
        }
    }

    return cpu_action;
}
