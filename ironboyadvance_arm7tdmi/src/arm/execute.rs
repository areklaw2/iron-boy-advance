use crate::{
    CpuAction, CpuState, Register,
    alu::{AluInstruction::*, *},
    barrel_shifter::{ShiftBy, ShiftType, asr, lsl, lsr, ror},
    cpu::{Arm7tdmiCpu, LR, PC},
    memory::{MemoryAccess, MemoryInterface},
};

use crate::arm::ArmInstruction;

pub fn execute_branch_and_exchange<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let value = cpu.register(instruction.rn() as usize);
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

pub fn execute_data_processing<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let mut cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
    let rn = instruction.rn() as usize;
    let operand1 = cpu.register(rn);
    let mut carry = cpu.cpsr().carry();

    let operand2 = match instruction.is_immediate_operand() {
        true => {
            let rotate = 2 * instruction.rotate();
            let immediate = instruction.immediate();
            immediate.rotate_right(rotate)
        }
        false => {
            let rm = instruction.rm();
            let rm_value = cpu.register(rm as usize);
            let shift_by = instruction.shift_by();
            let shift_amount = match shift_by {
                ShiftBy::Immediate => instruction.shift_amount(),
                ShiftBy::Register => {
                    // if rn == Register::R15 || rm == Register::R15 {
                    //     cpu.advance_pc_arm();
                    //     cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential);
                    // }
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

    let s = instruction.sets_condition();
    let opcode = instruction.opcode();
    let result = match opcode {
        AND => and(cpu, s, operand1, operand2, carry),
        EOR => eor(cpu, s, operand1, operand2, carry),
        SUB => sub(cpu, s, operand1, operand2),
        RSB => sub(cpu, s, operand2, operand1),
        ADD => add(cpu, s, operand1, operand2),
        ADC => adc(cpu, s, operand1, operand2),
        SBC => sbc(cpu, s, operand1, operand2),
        RSC => sbc(cpu, s, operand2, operand1),
        TST => and(cpu, s, operand1, operand2, carry),
        TEQ => eor(cpu, s, operand1, operand2, carry),
        CMP => sub(cpu, s, operand1, operand2),
        CMN => add(cpu, s, operand1, operand2),
        ORR => orr(cpu, s, operand1, operand2, carry),
        MOV => mov(cpu, s, operand2, carry),
        BIC => bic(cpu, s, operand1, operand2, carry),
        MVN => mvn(cpu, s, operand2, carry),
    };

    let rd = instruction.rd() as usize;
    if s && rd == PC {
        let spsr = cpu.spsr();
        //cpu.change_mode(spsr.cpu_mode());
        cpu.set_cpsr(spsr);
    }

    match opcode {
        TST | TEQ | CMP | CMN => {}
        _ => {
            cpu.set_register(rd, result);
            if rd == PC {
                cpu.refill_pipeline();
                cpu_action = CpuAction::PipelineFlush
            }
        }
    }

    cpu_action
}
