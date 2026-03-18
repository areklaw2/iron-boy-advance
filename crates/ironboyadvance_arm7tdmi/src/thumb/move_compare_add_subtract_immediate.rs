use crate::{
    BitOps, CpuAction, LoRegister, MovCmpAddSubImmediateOpcode,
    alu::{add, cmp, mov, sub},
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct MoveCompareAddSubtractImmediate {
    rd: LoRegister,
    offset: u16,
    opcode: u16,
}

impl MoveCompareAddSubtractImmediate {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(8..=10).into(),
            offset: value.bits(0..=7),
            opcode: value.bits(11..=12),
        }
    }
}

impl Instruction for MoveCompareAddSubtractImmediate {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        use MovCmpAddSubImmediateOpcode::*;
        let rd = self.rd as usize;
        let operand1 = cpu.register(rd);
        let offset = self.offset;
        let opcode: MovCmpAddSubImmediateOpcode = self.opcode.into();
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

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd;
        let offset = self.offset;
        let opcode = MovCmpAddSubImmediateOpcode::from(self.opcode);
        format!("{} {},#{}", opcode, rd, offset)
    }
}
