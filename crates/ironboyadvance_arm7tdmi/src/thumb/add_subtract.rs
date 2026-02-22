use crate::BitOps;

use crate::{
    CpuAction, LoRegister,
    alu::{add, sub},
    cpu::Arm7tdmiCpu,
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct AddSubtract {
    value: u16,
}

thumb_instruction!(AddSubtract);

impl AddSubtract {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
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

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
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

    #[inline]
    pub fn rd(&self) -> LoRegister {
        self.value.bits(0..=2).into()
    }

    #[inline]
    pub fn rs(&self) -> LoRegister {
        self.value.bits(3..=5).into()
    }

    #[inline]
    pub fn rn(&self) -> LoRegister {
        self.value.bits(6..=8).into()
    }

    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(6..=8)
    }

    #[inline]
    pub fn is_immediate(&self) -> bool {
        self.value.bit(10)
    }

    #[inline]
    pub fn opcode(&self) -> u16 {
        self.value.bit(9) as u16
    }
}
