use crate::{
    BitOps, CpuAction,
    cpu::{Arm7tdmiCpu, Instruction, SP},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct AddOffsetToSp {
    offset: u16,
    signed: bool,
}

impl AddOffsetToSp {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            offset: value.bits(0..=6),
            signed: value.bit(7),
        }
    }
}

impl Instruction for AddOffsetToSp {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = self.offset * 4;
        let sp_value = cpu.register(SP);
        let value = match self.signed {
            true => sp_value.wrapping_sub(offset as u32),
            false => sp_value.wrapping_add(offset as u32),
        };
        cpu.set_register(SP, value);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        let signed = if self.signed { "-" } else { "" };
        format!("ADD sp, {}{}", signed, offset)
    }
}
