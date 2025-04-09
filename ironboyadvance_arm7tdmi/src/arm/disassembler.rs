use crate::{CpuMode, alu::AluInstruction, barrel_shifter::ShiftBy, cpu::Arm7tdmiCpu, memory::MemoryInterface};

use super::ArmInstruction;

pub fn disassemble_bx(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let rn = instruction.rn();
    format!("BX{cond} {rn}")
}

pub fn disassemble_b_bl(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let link = if instruction.link() { "L" } else { "" };
    let expression = instruction.offset();
    format!("B{link}{cond} 0x{expression:08X}")
}

pub fn disassamble_data_processing(instruction: &ArmInstruction) -> String {
    use AluInstruction::*;
    let cond = instruction.cond();
    let opcode = instruction.opcode();
    let s = if instruction.sets_flags() { "S" } else { "" };
    let rd = instruction.rd();
    let rn = instruction.rn();
    let operand_2 = match instruction.is_immediate_operand() {
        true => {
            let rotate = 2 * instruction.rotate();
            let immediate = instruction.immediate();
            format!("0x{:08X}", immediate.rotate_right(rotate))
        }
        false => {
            let rm = instruction.rm();
            let shift_type = instruction.shift_type();
            match instruction.shift_by() {
                ShiftBy::Register => {
                    format!("{},{} {}", rm, shift_type, instruction.rs())
                }
                ShiftBy::Immediate => {
                    format!("{},{} #{}", rm, shift_type, instruction.shift_amount())
                }
            }
        }
    };

    match opcode {
        MOV | MVN => format!("{opcode}{cond}{s} {rd},{operand_2}"),
        CMP | CMN | TEQ | TST => format!("{opcode}{cond} {rn},{operand_2}"),
        _ => format!("{opcode}{cond}{s} {rd},{rn},{operand_2}"),
    }
}

pub fn disassemble_psr_transfer<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> String {
    //msr
    let cond = instruction.cond();
    let rd = instruction.rd();
    let psr = match instruction.is_spsr() {
        false => "CPSR",
        true => match cpu.cpsr().mode() {
            CpuMode::User | CpuMode::System => "CPSR",
            CpuMode::Fiq => "SPSR_fiq",
            CpuMode::Supervisor => "SPSR_svc",
            CpuMode::Abort => "SPSR_abt",
            CpuMode::Irq => "SPSR_irq",
            CpuMode::Undefined => "SPSR_und",
        },
    };

    format!("MRS{} {},{}", cond, rd, psr)
}
