use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::LoadStoreRegisterOffset};

impl Dissasemble for LoadStoreRegisterOffset {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let byte = if self.byte() { "B" } else { "" };
        let ro = self.ro();
        let rb = self.rb();
        let rd = self.rd();

        match self.load() {
            true => format!("LDR{} {}, [{},{}]", byte, rd, rb, ro),
            false => format!("STR{} {}, [{},{}]", byte, rd, rb, ro),
        }
    }
}
