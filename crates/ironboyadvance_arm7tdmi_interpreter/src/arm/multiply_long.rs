use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    alu::*,
    arm::MultiplyLong,
    cpu::{Arm7tdmiCpu, PC},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for MultiplyLong {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
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

        let mut result = match self.unsigned() {
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
}
