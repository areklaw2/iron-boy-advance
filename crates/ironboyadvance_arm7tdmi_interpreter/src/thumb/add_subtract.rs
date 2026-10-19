use crate::Execute;
use ironboyadvance_arm7tdmi::{CpuAction, alu::*, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::AddSubtract};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for AddSubtract {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let operand1 = cpu.register(self.rs() as usize);
        let operand2 = match self.is_immediate() {
            true => self.offset() as u32,
            false => cpu.register(self.rn() as usize),
        };

        let result = match self.opcode() != 0 {
            true => sub(cpu, true, operand1, operand2),
            false => add(cpu, true, operand1, operand2),
        };

        cpu.set_register(rd, result);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}
