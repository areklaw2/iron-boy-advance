use crate::{Dissasemble, arm::MultiplyLong, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for MultiplyLong {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let s = if self.sets_flags() { "S" } else { "" };
        let rd_hi = self.rd_hi();
        let rd_lo = self.rd_lo();
        let rm = self.rm();
        let rs = self.rs();
        let unsigned = self.unsigned();
        let accumulate = self.accumulate();
        match (unsigned, accumulate) {
            (true, false) => format!("UMULL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
            (true, true) => format!("UMLAL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
            (false, false) => format!("SMULL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
            (false, true) => format!("SMLAL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
        }
    }
}
