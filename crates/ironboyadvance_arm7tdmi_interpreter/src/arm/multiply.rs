use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    alu::*,
    arm::Multiply,
    cpu::{Arm7tdmiCpu, PC},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for Multiply {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rm = self.rm() as usize;
        let rs = self.rs() as usize;
        let rn = self.rn() as usize;

        let mut operand1 = cpu.register(rm);
        if rm == PC {
            operand1 += 4
        }
        let mut operand2 = cpu.register(rs);
        if rs == PC {
            operand2 += 4
        }

        let mut result = operand1.wrapping_mul(operand2);
        let multiplier_cycles = multiplier_array_cycles(operand2);
        for _ in 0..multiplier_cycles {
            cpu.idle_cycle();
        }

        if self.accumulate() {
            let mut accumulator = cpu.register(rn);
            if rn == PC {
                accumulator += 4
            }
            result = result.wrapping_add(accumulator);
            cpu.idle_cycle();
        };

        if self.sets_flags() {
            cpu.cpsr_mut().set_negative(result >> 31 != 0);
            cpu.cpsr_mut().set_zero(result == 0);
        }

        cpu.set_register(rd, result);
        match rd == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }
}
