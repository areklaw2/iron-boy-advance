use getset::CopyGetters;

use crate::{
    BitOps, Condition, CpuAction, Register,
    cpu::{Arm7tdmiCpu, Instruction, PC},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy, CopyGetters)]
pub struct SingleDataSwap {
    #[getset(get_copy = "pub(crate)")]
    cond: Condition,
    rn: Register,
    rd: Register,
    rm: Register,
    byte: bool,
}

impl SingleDataSwap {
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            byte: value.bit(22),
        }
    }
}

impl Instruction for SingleDataSwap {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd as usize;
        let rn = self.rn as usize;
        let rm = self.rm as usize;

        let address = cpu.register(rn);
        let mut source = cpu.register(rm);
        if rm == PC {
            source += 4;
        }

        let value: u32;
        match self.byte {
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

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let byte = if self.byte { "B" } else { "" };
        let rd = self.rd;
        let rm = self.rm;
        let rn = self.rn;
        format!("SWP{}{} {},{},[{}]", cond, byte, rd, rm, rn)
    }
}
