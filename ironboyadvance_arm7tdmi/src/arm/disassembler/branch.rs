use crate::arm::ArmInstruction;

pub fn disassemble_branch_and_exchange(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let rn = instruction.rn();
    format!("BX{cond} {rn}")
}

pub fn disassemble_branch_and_branch_with_link(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let link = if instruction.link() { "L" } else { "" };
    let expression = instruction.offset();
    format!("B{link}{cond} {expression:08X}")
}
