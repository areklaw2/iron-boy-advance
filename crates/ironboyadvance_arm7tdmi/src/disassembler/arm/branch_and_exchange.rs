use crate::{Dissasemble, arm::BranchAndExchange, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for BranchAndExchange {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let rn = self.rn();
        format!("BX{cond} {rn}")
    }
}
