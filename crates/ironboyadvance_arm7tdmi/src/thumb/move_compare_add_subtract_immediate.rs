use crate::BitOps;

use crate::{
    CpuAction, LoRegister, MovCmpAddSubImmediateOpcode,
    alu::{add, cmp, mov, sub},
    cpu::Arm7tdmiCpu,
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct MoveCompareAddSubtractImmediate {
    value: u16,
}

thumb_instruction!(MoveCompareAddSubtractImmediate);

impl MoveCompareAddSubtractImmediate {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
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

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd();
        let offset = self.offset();
        let opcode = MovCmpAddSubImmediateOpcode::from(self.opcode());
        format!("{} {},#{}", opcode, rd, offset)
    }

    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(0..=7)
    }

    #[inline]
    pub fn rd(&self) -> LoRegister {
        self.value.bits(8..=10).into()
    }

    #[inline]
    pub fn opcode(&self) -> u16 {
        self.value.bits(11..=12)
    }
}
