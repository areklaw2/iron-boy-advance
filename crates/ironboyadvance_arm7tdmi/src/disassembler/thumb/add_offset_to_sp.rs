use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::AddOffsetToSp};

impl Dissasemble for AddOffsetToSp {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        let signed = if self.signed() { "-" } else { "" };
        format!("ADD sp, {}{}", signed, offset)
    }
}
