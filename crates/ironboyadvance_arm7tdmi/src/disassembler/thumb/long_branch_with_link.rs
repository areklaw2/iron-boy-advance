use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::LongBranchWithLink};

impl Dissasemble for LongBranchWithLink {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        let high = if self.high() { "hi" } else { "lo" };
        format!("BL #{}({})", offset, high)
    }
}
