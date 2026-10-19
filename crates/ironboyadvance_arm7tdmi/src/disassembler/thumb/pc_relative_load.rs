use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::PcRelativeLoad};

impl Dissasemble for PcRelativeLoad {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd();
        let offset = self.offset();
        format!("LDR {},[PC, #{}]", rd, offset)
    }
}
