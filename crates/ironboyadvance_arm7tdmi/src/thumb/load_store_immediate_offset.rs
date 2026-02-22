use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct LoadStoreImmediateOffset {
    value: u16,
}

thumb_instruction!(LoadStoreImmediateOffset);

impl Instruction for LoadStoreImmediateOffset {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = if self.byte() { self.offset() } else { self.offset() * 4 };
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(immediate as u32);

        let rd = self.rd() as usize;
        let byte = self.byte();
        let load = self.load();
        match (load, byte) {
            (false, false) => {
                let value = cpu.register(rd);
                cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
            }
            (false, true) => {
                let value = cpu.register(rd);
                cpu.store_8(address, value as u8, MemoryAccess::NonSequential as u8);
            }
            (true, false) => {
                let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            (true, true) => {
                let value = cpu.load_8(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let byte = if self.byte() { "B" } else { "" };
        let offset = self.offset();
        let rb = self.rb();
        let rd = self.rd();

        match self.load() {
            true => format!("LDR{} {}, [{},#{}]", byte, rd, rb, offset),
            false => format!("STR{} {}, [{},#{}]", byte, rd, rb, offset),
        }
    }
}

impl LoadStoreImmediateOffset {
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

    #[inline]
    pub fn byte(&self) -> bool {
        self.value.bit(12)
    }
}
