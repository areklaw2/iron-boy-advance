use ironboyadvance_utils::bit::BitOps;

use crate::{
    CpuAction, Register,
    alu::multiplier_array_cycles,
    arm::arm_instruction,
    cpu::{Arm7tdmiCpu, PC},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct Multiply {
    value: u32,
}

arm_instruction!(Multiply);

impl Multiply {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
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

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let s = if self.sets_flags() { "S" } else { "" };
        let rd = self.rd();
        let rm = self.rm();
        let rs = self.rs();
        let rn = self.rn();
        match self.accumulate() {
            true => format!("MLA{}{} {},{},{},{}", cond, s, rd, rm, rs, rn),
            false => format!("MUL{}{} {},{},{}", cond, s, rd, rm, rs),
        }
    }

    #[inline]
    pub fn rn(&self) -> Register {
        self.value.bits(12..=15).into()
    }

    #[inline]
    pub fn rd(&self) -> Register {
        self.value.bits(16..=19).into()
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
}
