use ironboyadvance_utils::bit::BitOps;

use crate::{
    Condition, CpuAction, Register,
    cpu::{Arm7tdmiCpu, PC},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct SingleDataSwap {
    value: u32,
}

impl SingleDataSwap {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn execute_single_data_swap<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rn = self.rn() as usize;
        let rm = self.rm() as usize;

        let address = cpu.register(rn);
        let mut source = cpu.register(rm);
        if rm == PC {
            source += 4;
        }

        let value: u32;
        match self.byte() {
            true => {
                value = cpu.load_8(address, MemoryAccess::NonSequential as u8);
                cpu.store_8(address, source as u8, MemoryAccess::NonSequential | MemoryAccess::Lock);
            }
            false => {
                value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.store_32(address, source, MemoryAccess::NonSequential | MemoryAccess::Lock);
            }
        };

        cpu.idle_cycle();
        cpu.set_register(rd, value);
        match rd == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }

    pub fn disassemble_single_data_swap(&self) -> String {
        let cond = self.cond();
        let byte = if self.byte() { "B" } else { "" };
        let rd = self.rd();
        let rm = self.rm();
        let rn = self.rn();
        format!("SWP{}{} {},{},[{}]", cond, byte, rd, rm, rn)
    }

    #[inline]
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }

    #[inline]
    pub fn rn(&self) -> Register {
        self.value.bits(16..=19).into()
    }

    #[inline]
    pub fn rd(&self) -> Register {
        self.value.bits(12..=15).into()
    }

    #[inline]
    pub fn rm(&self) -> Register {
        self.value.bits(0..=3).into()
    }

    #[inline]
    pub fn byte(&self) -> bool {
        self.value.bit(22)
    }
}
