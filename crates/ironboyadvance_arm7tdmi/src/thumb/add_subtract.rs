use crate::{
    BitOps, CpuAction, LoRegister,
    alu::{add, sub},
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct AddSubtract {
    rd: LoRegister,
    rs: LoRegister,
    rn: LoRegister,
    offset: u16,
    is_immediate: bool,
    opcode: u16,
}

impl AddSubtract {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rs: value.bits(3..=5).into(),
            rn: value.bits(6..=8).into(),
            offset: value.bits(6..=8),
            is_immediate: value.bit(10),
            opcode: value.bit(9) as u16,
        }
    }
}

impl Instruction for AddSubtract {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd as usize;
        let operand1 = cpu.register(self.rs as usize);
        let operand2 = match self.is_immediate {
            true => self.offset as u32,
            false => cpu.register(self.rn as usize),
        };

        let result = match self.opcode != 0 {
            true => sub(cpu, true, operand1, operand2),
            false => add(cpu, true, operand1, operand2),
        };

        cpu.set_register(rd, result);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rs = self.rs;
        let rd = self.rd;
        let is_immediate = self.is_immediate;
        let operand = match is_immediate {
            true => format!("#{}", self.offset),
            false => format!("{}", self.rn),
        };
        let opcode = match self.opcode != 0 {
            true => "SUB",
            false => "ADD",
        };
        format!("{} {},{},{}", opcode, rd, rs, operand)
    }
}
