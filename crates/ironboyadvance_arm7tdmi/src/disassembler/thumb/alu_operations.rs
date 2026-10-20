use crate::{AluOperationsOpcode, Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::AluOperations};

impl Dissasemble for AluOperations {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd();
        let rs = self.rs();
        let opcode = AluOperationsOpcode::from(self.opcode());
        format!("{} {},{}", opcode, rd, rs)
    }
}
