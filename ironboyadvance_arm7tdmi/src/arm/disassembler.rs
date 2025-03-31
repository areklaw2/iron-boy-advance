use crate::{alu::AluInstruction, barrel_shifter::ShiftBy};

use super::ArmInstruction;

pub fn disassemble_branch_and_exchange(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let rn = instruction.rn();
    format!("BX{cond} {rn}")
}

pub fn disassemble_branch_and_branch_with_link(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let link = if instruction.link() { "L" } else { "" };
    let expression = instruction.offset();
    format!("B{link}{cond} 0x{expression:08X}")
}

pub fn disassamble_data_processing(instruction: &ArmInstruction) -> String {
    use AluInstruction::*;
    let cond = instruction.cond();
    let opcode = instruction.opcode();
    let s = if instruction.sets_condition() { "S" } else { "" };
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
