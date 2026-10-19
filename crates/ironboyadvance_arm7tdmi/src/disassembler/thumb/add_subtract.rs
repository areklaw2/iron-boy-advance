use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::AddSubtract};

impl Dissasemble for AddSubtract {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rs = self.rs();
        let rd = self.rd();
        let is_immediate = self.is_immediate();
        let operand = match is_immediate {
            true => format!("#{}", self.offset()),
            false => format!("{}", self.rn()),
        };
        let opcode = match self.opcode() != 0 {
            true => "SUB",
            false => "ADD",
        };
        format!("{} {},{},{}", opcode, rd, rs, operand)
    }
}
