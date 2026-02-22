use crate::BitOps;

use crate::{
    CpuAction, LoRegister,
    cpu::Arm7tdmiCpu,
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct LoadStoreRegisterOffset {
    value: u16,
}

thumb_instruction!(LoadStoreRegisterOffset);

impl LoadStoreRegisterOffset {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let ro_value = cpu.register(self.ro() as usize);
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(ro_value);

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

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let byte = if self.byte() { "B" } else { "" };
        let ro = self.ro();
        let rb = self.rb();
        let rd = self.rd();

        match self.load() {
            true => format!("LDR{} {}, [{},{}]", byte, rd, rb, ro),
            false => format!("STR{} {}, [{},{}]", byte, rd, rb, ro),
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
    pub fn ro(&self) -> LoRegister {
        self.value.bits(6..=8).into()
    }

    #[inline]
    pub fn byte(&self) -> bool {
        self.value.bit(10)
    }

    #[inline]
    pub fn load(&self) -> bool {
        self.value.bit(11)
    }
}
