use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::LoadAddress};

impl Dissasemble for LoadAddress {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        let rd = self.rd();
        let sp = if self.sp() { "sp" } else { "pc" };
        format!("ADD {},{},{}", rd, sp, offset)
    }
}
