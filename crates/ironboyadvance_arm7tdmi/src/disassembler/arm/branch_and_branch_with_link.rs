use crate::{Dissasemble, arm::BranchAndBranchWithLink, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for BranchAndBranchWithLink {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let link = if self.link() { "L" } else { "" };
        let expression = self.offset();
        format!("B{link}{cond} 0x{expression:08X}")
    }
}
