use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction, MovCmpAddSubImmediateOpcode, alu::*, cpu::Arm7tdmiCpu, memory::MemoryInterface,
    thumb::MoveCompareAddSubtractImmediate,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for MoveCompareAddSubtractImmediate {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        use MovCmpAddSubImmediateOpcode::*;
        let rd = self.rd() as usize;
        let operand1 = cpu.register(rd);
        let offset = self.offset();
        let opcode: MovCmpAddSubImmediateOpcode = self.opcode().into();
        let result = match opcode {
            MOV => mov(cpu, true, offset as u32, cpu.cpsr().carry()),
            CMP => cmp(cpu, true, operand1, offset as u32),
            ADD => add(cpu, true, operand1, offset as u32),
            SUB => sub(cpu, true, operand1, offset as u32),
        };

        if opcode != CMP {
            cpu.set_register(rd, result);
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}
