use super::ThumbInstruction;

pub fn disassemble_move_shifted_register(instruction: &ThumbInstruction) -> String {
    let opcode = instruction.opcode();
    let offset5 = instruction.offset5();
    let rs = instruction.rs();
    let rd = instruction.rd();
    format!("{} {},{},#{}", opcode, rd, rs, offset5)
}
