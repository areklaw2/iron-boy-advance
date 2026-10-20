use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::UnconditionalBranch};

impl Dissasemble for UnconditionalBranch {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        format!("B #{}", offset)
    }
}
