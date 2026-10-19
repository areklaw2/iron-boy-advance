use crate::{Dissasemble, arm::SoftwareInterrupt, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for SoftwareInterrupt {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let comment = self.comment();
        format!("SWI{} 0x{:08X}", cond, comment)
    }
}
