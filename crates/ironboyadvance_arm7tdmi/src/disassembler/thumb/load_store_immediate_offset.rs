use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::LoadStoreImmediateOffset};

impl Dissasemble for LoadStoreImmediateOffset {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let byte = if self.byte() { "B" } else { "" };
        let offset = self.offset();
        let rb = self.rb();
        let rd = self.rd();

        match self.load() {
            true => format!("LDR{} {}, [{},#{}]", byte, rd, rb, offset),
            false => format!("STR{} {}, [{},#{}]", byte, rd, rb, offset),
        }
    }
}
