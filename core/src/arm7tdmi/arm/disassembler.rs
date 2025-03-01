use super::ArmInstruction;

impl ArmInstruction {
    pub fn disassemble_branch_and_exchange(&self) -> String {
        let cond = self.cond();
        let rn = self.rn();
        format!("BX{cond} {rn}")
    }

    pub fn disassemble_branch_and_branch_with_link(&self) -> String {
        let cond = self.cond();
        let link = if self.link() { "L" } else { "" };
        let expression = "";
        format!("B{link}{cond} {expression}")
    }
}
