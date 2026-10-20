use crate::{Dissasemble, arm::Multiply, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for Multiply {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let s = if self.sets_flags() { "S" } else { "" };
        let rd = self.rd();
        let rm = self.rm();
        let rs = self.rs();
        let rn = self.rn();
        match self.accumulate() {
            true => format!("MLA{}{} {},{},{},{}", cond, s, rd, rm, rs, rn),
            false => format!("MUL{}{} {},{},{}", cond, s, rd, rm, rs),
        }
    }
}
