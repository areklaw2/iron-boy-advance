use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::SpRelativeLoadStore};

impl Dissasemble for SpRelativeLoadStore {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        let rd = self.rd();
        match self.load() {
            true => format!("LDR {}, [sp,#{}]", rd, offset),
            false => format!("STRH {}, [sp,#{}]", rd, offset),
        }
    }
}
