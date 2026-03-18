use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction, SP},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct SpRelativeLoadStore {
    rd: LoRegister,
    offset: u16,
    load: bool,
}

impl SpRelativeLoadStore {
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(8..=10).into(),
            offset: value.bits(0..=7),
            load: value.bit(11),
        }
    }
}

impl Instruction for SpRelativeLoadStore {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = self.offset * 4;
        let sp_value = cpu.register(SP);
        let address = sp_value.wrapping_add(immediate as u32);
        let rd = self.rd as usize;
        match self.load {
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

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        let rd = self.rd;
        match self.load {
            true => format!("LDR {}, [sp,#{}]", rd, offset),
            false => format!("STRH {}, [sp,#{}]", rd, offset),
        }
    }
}
