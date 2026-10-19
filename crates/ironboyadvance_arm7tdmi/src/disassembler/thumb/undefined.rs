use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::Undefined};

impl Dissasemble for Undefined {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        "Undefined".into()
    }
}
