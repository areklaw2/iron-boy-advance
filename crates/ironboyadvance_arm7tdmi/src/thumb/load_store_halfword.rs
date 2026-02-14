use ironboyadvance_utils::bit::BitOps;

use crate::{
    CpuAction, LoRegister,
    cpu::Arm7tdmiCpu,
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct LoadStoreHalfword {
    value: u16,
}

thumb_instruction!(LoadStoreHalfword);

impl LoadStoreHalfword {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = self.offset() * 2;
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(immediate as u32);
        let rd = self.rd() as usize;
        match self.load() {
            true => {
                let value = cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            false => {
                let value = cpu.register(rd);
                cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        let rb = self.rb();
        let rd = self.rd();
        match self.load() {
            true => format!("LDRH {}, [{},#{}]", rd, rb, offset),
            false => format!("STRH {}, [{},#{}]", rd, rb, offset),
        }
    }

    #[inline]
    pub fn rd(&self) -> LoRegister {
        self.value.bits(0..=2).into()
    }

    #[inline]
    pub fn rb(&self) -> LoRegister {
        self.value.bits(3..=5).into()
    }

    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(6..=10)
    }

    #[inline]
    pub fn load(&self) -> bool {
        self.value.bit(11)
    }
}
