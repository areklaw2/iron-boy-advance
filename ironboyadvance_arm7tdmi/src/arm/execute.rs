use crate::{
    CpuAction, CpuState,
    alu::{AluInstruction, AluInstruction::*, ShiftBy},
    cpu::{Arm7tdmiCpu, LR},
    memory::MemoryInterface,
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
    let rn = instruction.rn();

    let s = instruction.sets_condition();
    let rd = instruction.rd();
    let opcode = instruction.opcode();
    let operand_2 = match instruction.is_immediate_operand() {
        true => {
            let rotate = instruction.rotate();
            let immediate = instruction.immediate();
            immediate.rotate_right(rotate).to_string()
        }
        false => {
            let rm = instruction.rm();
            let shift_type = instruction.shift_type();
            match instruction.shift_by() {
                ShiftBy::Register => {
                    format!("{},{} {}", rm, shift_type, instruction.rs())
                }
                ShiftBy::Amount => {
                    format!("{},{} {}", rm, shift_type, instruction.shift_amount())
                }
            }
        }
    };

    match opcode {
        AND => todo!(),
        EOR => todo!(),
        SUB => todo!(),
        RSB => todo!(),
        ADD => todo!(),
        ADC => todo!(),
        SBC => todo!(),
        RSC => todo!(),
        TST => todo!(),
        TEQ => todo!(),
        CMP => todo!(),
        CMN => todo!(),
        ORR => todo!(),
        MOV => todo!(),
        BIC => todo!(),
        MVN => todo!(),
    }
}
