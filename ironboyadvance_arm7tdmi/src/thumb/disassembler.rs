use super::ThumbInstruction;

pub fn disassemble_move_shifted_register(instruction: &ThumbInstruction) -> String {
    let shift_type = instruction.opcode();
    let offset5 = instruction.offset5();
    let rs = instruction.rs();
    let rd = instruction.rd();
    format!("{} {},{},#{}", shift_type, rd, rs, offset5)
}

pub fn disassemble_add_subtract(instruction: &ThumbInstruction) -> String {
    let rs = instruction.rs();
    let rd = instruction.rd();
    let is_immediate = instruction.is_immediate();
    let operand = match is_immediate {
        true => format!("#{}", instruction.offset3()),
        false => format!("{}", instruction.rn()),
    };
    let opcode = match instruction.opcode() != 0 {
        true => "SUB",
        false => "ADD",
    };
    format!("{} {},{},{}", opcode, rd, rs, operand)
}
