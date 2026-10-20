use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::SoftwareInterrupt};

impl Dissasemble for SoftwareInterrupt {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        format!("SWI #{}", offset)
    }
}
