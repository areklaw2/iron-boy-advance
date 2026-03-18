use getset::CopyGetters;

use crate::{
    BitOps, Condition, CpuAction, CpuMode, Register,
    barrel_shifter::ror,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
    psr::ProgramStatusRegister,
};

#[derive(Debug, Clone, Copy, CopyGetters)]
pub struct PsrTransfer {
    #[getset(get_copy = "pub(crate)")]
    cond: Condition,
    is_mrs: bool,
    psr_mask: u32,
    rd: Register,
    rm: Register,
    is_immediate: bool,
    rotate: u32,
    immediate: u32,
    is_spsr: bool,
}

impl PsrTransfer {
    #[inline]
    pub fn new(value: u32) -> Self {
        let field_bits = value.bits(16..=19);
        let mut psr_mask = 0u32;
        if field_bits & 0b1000 != 0 {
            psr_mask |= 0xFF000000;
        }
        if field_bits & 0b0100 != 0 {
            psr_mask |= 0xFF0000;
        }
        if field_bits & 0b0010 != 0 {
            psr_mask |= 0xFF00;
        }
        if field_bits & 0b0001 != 0 {
            psr_mask |= 0xFF;
        }

        Self {
            cond: value.bits(28..=31).into(),
            is_mrs: value.bits(16..=21) as u8 == 0xF,
            psr_mask,
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            is_immediate: value.bit(25),
            rotate: value.bits(8..=11),
            immediate: value.bits(0..=7),
            is_spsr: value.bit(22),
        }
    }
}

impl Instruction for PsrTransfer {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let is_spsr = self.is_spsr;
        match self.is_mrs {
            true => {
                let rd = self.rd as usize;
                let psr = match is_spsr {
                    false => *cpu.cpsr(),
                    true => cpu.spsr(),
                };
                cpu.set_register(rd, psr.into_bits());
            }
            false => {
                let mask = self.psr_mask;

                let mut operand = match self.is_immediate {
                    false => cpu.register(self.rm as usize),
                    true => {
                        let mut carry = cpu.cpsr().carry();
                        let rotate = 2 * self.rotate;
                        let immediate = self.immediate;
                        ror(immediate, rotate, &mut carry, false)
                    }
                };

                match is_spsr {
                    false => {
                        // User mode can only change flags
                        if cpu.cpsr().mode() == CpuMode::User {
                            let mask = mask & 0xFF000000;
                            let bits = (cpu.cpsr().into_bits() & !mask) | (operand & mask);
                            cpu.set_cpsr(ProgramStatusRegister::from_bits(bits));
                        } else {
                            // Make sure operand has the 4th bit
                            if mask & 0xFF != 0 {
                                operand |= 0x10;
                            }

                            let bits = (cpu.cpsr().into_bits() & !mask) | (operand & mask);
                            cpu.set_cpsr(ProgramStatusRegister::from_bits(bits));
                        }
                    }
                    true => {
                        if cpu.cpsr().mode() != CpuMode::User && cpu.cpsr().mode() != CpuMode::System {
                            let bits = (cpu.spsr().into_bits() & !mask) | (operand & mask);
                            cpu.set_spsr(ProgramStatusRegister::from_bits(bits));
                        }
                    }
                }
            }
        }
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let is_spsr = self.is_spsr;
        let psr = match is_spsr {
            false => "CPSR",
            true => match cpu.cpsr().mode() {
                CpuMode::User | CpuMode::System => "CPSR",
                CpuMode::Fiq => "SPSR_fiq",
                CpuMode::Supervisor => "SPSR_svc",
                CpuMode::Abort => "SPSR_abt",
                CpuMode::Irq => "SPSR_irq",
                CpuMode::Undefined => "SPSR_und",
                CpuMode::Invalid => panic!("invalid mode"),
            },
        };

        match self.is_mrs {
            true => {
                let rd = self.rd as usize;
                format!("MRS{} {},{}", cond, rd, psr)
            }
            false => {
                let operand = match self.is_immediate {
                    false => format!("{}", self.rm),
                    true => {
                        let rotate = 2 * self.rotate;
                        let immediate = self.immediate;
                        let expression = immediate.rotate_right(rotate);
                        format!("0x{:08X}", expression)
                    }
                };

                match is_spsr {
                    false => {
                        if cpu.cpsr().mode() == CpuMode::User {
                            return format!("MSR{} {}_flg,{}", cond, psr, operand);
                        }
                        format!("MSR{} {}_all,{}", cond, psr, operand)
                    }
                    true => format!("MSR{} {},{}", cond, psr, operand),
                }
            }
        }
    }
}
