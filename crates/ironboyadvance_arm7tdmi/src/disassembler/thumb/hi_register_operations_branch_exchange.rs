use crate::{
    Dissasemble, HiRegOpsBxOpcode, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::HiRegisterOperationsBranchExchange,
};

impl Dissasemble for HiRegisterOperationsBranchExchange {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let destination = match self.h1() {
            true => self.hd().to_string(),
            false => self.rd().to_string(),
        };

        let source = match self.h2() {
            true => self.hs().to_string(),
            false => self.rs().to_string(),
        };

        let opcode = HiRegOpsBxOpcode::from(self.opcode());
        format!("{} {},{}", opcode, destination, source)
    }
}
