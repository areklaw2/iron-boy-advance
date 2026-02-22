use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction, SP},
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct LoadAddress {
    value: u16,
}

thumb_instruction!(LoadAddress);

impl Instruction for LoadAddress {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let offset = self.offset() * 4;
        let value = match self.sp() {
            true => cpu.register(SP).wrapping_add(offset as u32),
            false => (cpu.pc() & !0b10).wrapping_add(offset as u32),
        };
        cpu.set_register(rd, value);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        let rd = self.rd();
        let sp = if self.sp() { "sp" } else { "pc" };
        format!("ADD {},{},{}", rd, sp, offset)
    }
}

impl LoadAddress {
    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(0..=7)
    }

    #[inline]
    pub fn rd(&self) -> LoRegister {
        self.value.bits(8..=10).into()
    }

    #[inline]
    pub fn sp(&self) -> bool {
        self.value.bit(11)
    }
}
