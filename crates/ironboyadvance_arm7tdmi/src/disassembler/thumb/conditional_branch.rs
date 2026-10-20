use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::ConditionalBranch};

impl Dissasemble for ConditionalBranch {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let offset = self.offset();
        format!("B{} #{}", cond, offset)
    }
}
