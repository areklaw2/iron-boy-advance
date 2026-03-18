use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction, SP},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct LoadAddress {
    rd: LoRegister,
    offset: u16,
    sp: bool,
}

impl LoadAddress {
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(8..=10).into(),
            offset: value.bits(0..=7),
            sp: value.bit(11),
        }
    }
}

impl Instruction for LoadAddress {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd as usize;
        let offset = self.offset * 4;
        let value = match self.sp {
            true => cpu.register(SP).wrapping_add(offset as u32),
            false => (cpu.pc() & !0b10).wrapping_add(offset as u32),
        };
        cpu.set_register(rd, value);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        let rd = self.rd;
        let sp = if self.sp { "sp" } else { "pc" };
        format!("ADD {},{},{}", rd, sp, offset)
    }
}
