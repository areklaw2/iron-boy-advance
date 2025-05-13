use crate::alu::{AluOperationsOpcode, MovCmpAddSubImmediateOpcode};

use super::ThumbInstruction;

pub fn disassemble_move_shifted_register(instruction: &ThumbInstruction) -> String {
    let shift_type = instruction.opcode();
    let offset5 = instruction.offset();
    let rs = instruction.rs();
    let rd = instruction.rd();
    format!("{} {},{},#{}", shift_type, rd, rs, offset5)
}

pub fn disassemble_add_subtract(instruction: &ThumbInstruction) -> String {
    let rs = instruction.rs();
    let rd = instruction.rd();
    let is_immediate = instruction.is_immediate();
    let operand = match is_immediate {
        true => format!("#{}", instruction.offset()),
        false => format!("{}", instruction.rn()),
    };
    let opcode = match instruction.opcode() != 0 {
        true => "SUB",
        false => "ADD",
    };
    format!("{} {},{},{}", opcode, rd, rs, operand)
}

pub fn disassemble_move_compare_add_subtract_immediate(instruction: &ThumbInstruction) -> String {
    let rd = instruction.rd();
    let offset = instruction.offset();
    let opcode = MovCmpAddSubImmediateOpcode::from(instruction.opcode());
    format!("{} {},#{}", opcode, rd, offset)
}

pub fn disassemble_alu_operations(instruction: &ThumbInstruction) -> String {
    let rd = instruction.rd();
    let rs = instruction.rs();
    let opcode = AluOperationsOpcode::from(instruction.opcode());
    format!("{} {},{}", opcode, rd, rs)
}
