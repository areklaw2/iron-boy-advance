use crate::{Dissasemble, arm::SingleDataSwap, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for SingleDataSwap {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let byte = if self.byte() { "B" } else { "" };
        let rd = self.rd();
        let rm = self.rm();
        let rn = self.rn();
        format!("SWP{}{} {},{},[{}]", cond, byte, rd, rm, rn)
    }
}
