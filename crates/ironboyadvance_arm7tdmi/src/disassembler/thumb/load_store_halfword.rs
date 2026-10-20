use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::LoadStoreHalfword};

impl Dissasemble for LoadStoreHalfword {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        let rb = self.rb();
        let rd = self.rd();
        match self.load() {
            true => format!("LDRH {}, [{},#{}]", rd, rb, offset),
            false => format!("STRH {}, [{},#{}]", rd, rb, offset),
        }
    }
}
