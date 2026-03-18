use getset::CopyGetters;

use crate::{
    AluOperationsOpcode, BitOps, CpuAction, LoRegister,
    alu::*,
    barrel_shifter::{asr, lsl, lsr, ror},
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy, CopyGetters)]
pub struct AluOperations {
    rd: LoRegister,
    rs: LoRegister,
    #[getset(get_copy = "pub(crate)")]
    opcode: u16,
}

impl AluOperations {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rs: value.bits(3..=5).into(),
            opcode: value.bits(6..=9),
        }
    }
}

impl Instruction for AluOperations {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        use AluOperationsOpcode::*;
        let rd = self.rd as usize;
        let operand1 = cpu.register(rd);
        let mut operand2 = cpu.register(self.rs as usize);
        let mut carry = cpu.cpsr().carry();
        let opcode: AluOperationsOpcode = self.opcode.into();
        let mut access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);

        let result = match opcode {
            AND => and(cpu, true, operand1, operand2, carry),
            EOR => eor(cpu, true, operand1, operand2, carry),
            LSL => {
                operand2 &= 0xFF;
                let result = lsl(operand1, operand2, &mut carry);
                cpu.idle_cycle();
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                cpu.cpsr_mut().set_carry(carry);
                result
            }
            LSR => {
                operand2 &= 0xFF;
                let result = lsr(operand1, operand2, &mut carry, false);
                cpu.idle_cycle();
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                cpu.cpsr_mut().set_carry(carry);
                result
            }
            ASR => {
                operand2 &= 0xFF;
                let result = asr(operand1, operand2, &mut carry, false);
                cpu.idle_cycle();
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                cpu.cpsr_mut().set_carry(carry);
                result
            }
            ADC => adc(cpu, true, operand1, operand2),
            SBC => sbc(cpu, true, operand1, operand2),
            ROR => {
                operand2 &= 0xFF;
                let result = ror(operand1, operand2, &mut carry, false);
                cpu.idle_cycle();
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                cpu.cpsr_mut().set_carry(carry);
                result
            }
            TST => tst(cpu, true, operand1, operand2, carry),
            NEG => sub(cpu, true, 0, operand2),
            CMP => cmp(cpu, true, operand1, operand2),
            CMN => cmn(cpu, true, operand1, operand2),
            ORR => orr(cpu, true, operand1, operand2, carry),
            MUL => {
                let multiplier_cycles = multiplier_array_cycles(operand1);
                for _ in 0..multiplier_cycles {
                    cpu.idle_cycle();
                }
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                let result = operand1.wrapping_mul(operand2);
                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                result
            }
            BIC => bic(cpu, true, operand1, operand2, carry),
            MVN => mvn(cpu, true, operand2, carry),
        };

        if ![TST, CMP, CMN].contains(&opcode) {
            cpu.set_register(rd, result);
        }

        access
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd;
        let rs = self.rs;
        let opcode = AluOperationsOpcode::from(self.opcode);
        format!("{} {},{}", opcode, rd, rs)
    }
}
