use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct LoadStoreHalfword {
    rd: LoRegister,
    rb: LoRegister,
    offset: u16,
    load: bool,
}

impl LoadStoreHalfword {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rb: value.bits(3..=5).into(),
            offset: value.bits(6..=10),
            load: value.bit(11),
        }
    }
}

impl Instruction for LoadStoreHalfword {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = self.offset * 2;
        let rb_value = cpu.register(self.rb as usize);
        let address = rb_value.wrapping_add(immediate as u32);
        let rd = self.rd as usize;
        match self.load {
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

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        let rb = self.rb;
        let rd = self.rd;
        match self.load {
            true => format!("LDRH {}, [{},#{}]", rd, rb, offset),
            false => format!("STRH {}, [{},#{}]", rd, rb, offset),
        }
    }
}
