use crate::{
    CpuAction, CpuState, Register,
    alu::{
        AluInstruction::{self, *},
        sbc,
    },
    barrel_shifter::{ShiftBy, ShiftType, asr, lsl, lsr, ror},
    cpu::{Arm7tdmiCpu, LR},
    memory::{MemoryAccess, MemoryInterface},
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

pub fn execute_data_processing<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> CpuAction {
    let mut cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
    let rn = instruction.rn();
    let operand_1 = cpu.get_register(rn as usize);
    let mut carry = cpu.cpsr().carry();

    let operand_2 = match instruction.is_immediate_operand() {
        true => {
            let rotate = 2 * instruction.rotate();
            let immediate = instruction.immediate();
            immediate.rotate_right(rotate)
        }
        false => {
            let rm = instruction.rm();
            let rm_value = cpu.get_register(rm as usize);
            let shift_by = instruction.shift_by();
            let shift_amount = match shift_by {
                ShiftBy::Immediate => instruction.shift_amount(),
                ShiftBy::Register => {
                    // if rn == Register::R15 || rm == Register::R15 {
                    //     cpu.advance_pc_arm();
                    //     cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Nonsequential);
                    // }
                    cpu.idle_cycle();
                    cpu.get_register(instruction.rs() as usize) & 0xFF
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
    let rd = instruction.rd();
    let opcode = instruction.opcode();
    let flags: u8 = match opcode {
        AND => todo!(),
        EOR => todo!(),
        SUB => todo!(),
        RSB => todo!(),
        ADD => todo!(),
        ADC => todo!(),
        SBC => sbc(operand_1, operand_2, carry),
        RSC => todo!(),
        TST => todo!(),
        TEQ => todo!(),
        CMP => todo!(),
        CMN => todo!(),
        ORR => todo!(),
        MOV => todo!(),
        BIC => todo!(),
        MVN => todo!(),
    };

    if s {
        println!("set flags")
    }

    cpu_action
}
