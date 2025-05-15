use crate::{AluOperationsOpcode, HiRegOpsBxOpcode, MovCmpAddSubImmediateOpcode};

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

pub fn disassemble_hi_register_operations_branch_exchange(instruction: &ThumbInstruction) -> String {
    let destination = match instruction.h1() {
        true => instruction.hd().to_string(),
        false => instruction.rd().to_string(),
    };

    let source = match instruction.h2() {
        true => instruction.hs().to_string(),
        false => instruction.rs().to_string(),
    };

    let opcode = HiRegOpsBxOpcode::from(instruction.opcode());
    format!("{} {},{}", opcode, destination, source)
}

pub fn disassemble_pc_relative_load(instruction: &ThumbInstruction) -> String {
    let rd = instruction.rd();
    let offset = instruction.offset();
    format!("LDR {},[PC, #{}]", rd, offset)
}

pub fn disassemble_load_store_register_offset(instruction: &ThumbInstruction) -> String {
    let byte = if instruction.byte() { "B" } else { "" };
    let ro = instruction.ro();
    let rb = instruction.rb();
    let rd = instruction.rd();

    match instruction.load() {
        true => format!("LDR{} {}, [{},{}]", byte, rd, rb, ro),
        false => format!("STR{} {}, [{},{}]", byte, rd, rb, ro),
    }
}

pub fn disassemble_load_store_sign_extended_byte_halfword(instruction: &ThumbInstruction) -> String {
    let ro = instruction.ro();
    let rb = instruction.rb();
    let rd = instruction.rd();
    let signed = instruction.signed();
    let halfword = instruction.halfword();
    match (signed, halfword) {
        (false, false) => format!("STRH {}, [{},{}]", rd, rb, ro),
        (false, true) => format!("LDRH {}, [{},{}]", rd, rb, ro),
        (true, false) => format!("LDSB {}, [{},{}]", rd, rb, ro),
        (true, true) => format!("LDSH {}, [{},{}]", rd, rb, ro),
    }
}

pub fn disassemble_load_store_immediate_offset(instruction: &ThumbInstruction) -> String {
    let byte = if instruction.byte() { "B" } else { "" };
    let offset = instruction.offset();
    let rb = instruction.rb();
    let rd = instruction.rd();

    match instruction.load() {
        true => format!("LDR{} {}, [{},#{}]", byte, rd, rb, offset),
        false => format!("STR{} {}, [{},#{}]", byte, rd, rb, offset),
    }
}

pub fn disassemble_load_store_halfword(instruction: &ThumbInstruction) -> String {
    let offset = instruction.offset();
    let rb = instruction.rb();
    let rd = instruction.rd();
    match instruction.load() {
        true => format!("LDRH {}, [{},#{}]", rd, rb, offset),
        false => format!("STRH {}, [{},#{}]", rd, rb, offset),
    }
}

pub fn disassemble_sp_relative_load_store(instruction: &ThumbInstruction) -> String {
    let offset = instruction.offset();
    let rd = instruction.rd();
    match instruction.load() {
        true => format!("LDR {}, [sp,#{}]", rd, offset),
        false => format!("STRH {}, [sp,#{}]", rd, offset),
    }
}

pub fn disassemble_load_address(instruction: &ThumbInstruction) -> String {
    let offset = instruction.offset();
    let rd = instruction.rd();
    let sp = if instruction.sp() { "sp" } else { "pc" };
    format!("ADD {},{},{}", rd, sp, offset)
}

pub fn disassemble_add_offset_to_sp(instruction: &ThumbInstruction) -> String {
    let offset = instruction.offset();
    let signed = if instruction.signed() { "-" } else { "" };
    format!("ADD sp, {}{}", signed, offset)
}
