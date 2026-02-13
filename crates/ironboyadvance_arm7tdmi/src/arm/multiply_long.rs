use ironboyadvance_utils::bit::BitOps;

use crate::{
    Condition, CpuAction, Register,
    alu::multiplier_array_cycles,
    cpu::{Arm7tdmiCpu, PC},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct MultiplyLong {
    value: u32,
}

impl MultiplyLong {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd_lo = self.rd_lo() as usize;
        let rd_hi = self.rd_hi() as usize;
        let rm = self.rm() as usize;
        let rs = self.rs() as usize;

        let mut operand1 = cpu.register(rm);
        if rm == PC {
            operand1 += 4
        }
        let mut operand2 = cpu.register(rs);
        if rs == PC {
            operand2 += 4
        }

        let unsigned = self.unsigned();
        let mut result = match unsigned {
            true => (operand1 as i32 as i64).wrapping_mul(operand2 as i32 as i64) as u64,
            false => (operand1 as u64).wrapping_mul(operand2 as u64),
        };

        let multiplier_cycles = multiplier_array_cycles(operand2);
        for _ in 0..multiplier_cycles {
            cpu.idle_cycle();
        }

        if self.accumulate() {
            let mut accumulator_lo = cpu.register(rd_lo) as u64;
            if rd_lo == PC {
                accumulator_lo += 4
            }
            let mut accumulator_hi = cpu.register(rd_hi) as u64;
            if rd_hi == PC {
                accumulator_hi += 4
            }
            result = result.wrapping_add(accumulator_hi << 32 | accumulator_lo);
            cpu.idle_cycle();
        };

        let result_lo = (result & 0xFFFFFFFF) as u32;
        let result_hi = (result >> 32) as u32;
        if self.sets_flags() {
            cpu.cpsr_mut().set_negative(result_hi >> 31 != 0);
            cpu.cpsr_mut().set_zero(result == 0);
        }

        cpu.set_register(rd_lo, result_lo);
        cpu.set_register(rd_hi, result_hi);
        match rd_hi == PC || rd_lo == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }

    pub fn disassemble(&self) -> String {
        let cond = self.cond();
        let s = if self.sets_flags() { "S" } else { "" };
        let rd_hi = self.rd_hi();
        let rd_lo = self.rd_lo();
        let rm = self.rm();
        let rs = self.rs();
        let unsigned = self.unsigned();
        let accumulate = self.accumulate();
        match (unsigned, accumulate) {
            (true, false) => format!("UMULL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
            (true, true) => format!("UMLAL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
            (false, false) => format!("SMULL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
            (false, true) => format!("SMLAL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
        }
    }

    #[inline]
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }

    #[inline]
    pub fn rd_hi(&self) -> Register {
        self.value.bits(16..=19).into()
    }

    #[inline]
    pub fn rd_lo(&self) -> Register {
        self.value.bits(12..=15).into()
    }

    #[inline]
    pub fn rm(&self) -> Register {
        self.value.bits(0..=3).into()
    }

    #[inline]
    pub fn rs(&self) -> Register {
        self.value.bits(8..=11).into()
    }

    #[inline]
    pub fn sets_flags(&self) -> bool {
        self.value.bit(20)
    }

    #[inline]
    pub fn accumulate(&self) -> bool {
        self.value.bit(21)
    }

    #[inline]
    pub fn unsigned(&self) -> bool {
        self.value.bit(22)
    }
}
