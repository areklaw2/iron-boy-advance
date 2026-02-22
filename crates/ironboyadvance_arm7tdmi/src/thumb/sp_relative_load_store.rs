use crate::BitOps;

use crate::{
    CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, SP},
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct SpRelativeLoadStore {
    value: u16,
}

thumb_instruction!(SpRelativeLoadStore);

impl SpRelativeLoadStore {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = self.offset() * 4;
        let sp_value = cpu.register(SP);
        let address = sp_value.wrapping_add(immediate as u32);
        let rd = self.rd() as usize;
        match self.load() {
            true => {
                let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            false => {
                let value = cpu.register(rd);
                cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset();
        let rd = self.rd();
        match self.load() {
            true => format!("LDR {}, [sp,#{}]", rd, offset),
            false => format!("STRH {}, [sp,#{}]", rd, offset),
        }
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
    pub fn load(&self) -> bool {
        self.value.bit(11)
    }
}
