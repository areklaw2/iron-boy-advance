use crate::BitOps;

use crate::{
    CpuAction, CpuMode, Register,
    arm::arm_instruction,
    barrel_shifter::ror,
    cpu::Arm7tdmiCpu,
    memory::{MemoryAccess, MemoryInterface},
    psr::ProgramStatusRegister,
};

#[derive(Debug, Clone, Copy)]
pub struct PsrTransfer {
    value: u32,
}

arm_instruction!(PsrTransfer);

impl PsrTransfer {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let is_spsr = self.is_spsr();
        match self.value.bits(16..=21) as u8 == 0xF {
            true => {
                let rd = self.rd() as usize;
                let psr = match is_spsr {
                    false => *cpu.cpsr(),
                    true => cpu.spsr(),
                };
                cpu.set_register(rd, psr.into_bits());
            }
            false => {
                let mut mask = 0u32;
                if self.value.bit(19) {
                    mask |= 0xFF000000;
                }
                if self.value.bit(18) {
                    mask |= 0xFF0000;
                }
                if self.value.bit(17) {
                    mask |= 0xFF00;
                }
                if self.value.bit(16) {
                    mask |= 0xFF;
                }

                let mut operand = match self.is_immediate() {
                    false => cpu.register(self.rm() as usize),
                    true => {
                        let mut carry = cpu.cpsr().carry();
                        let rotate = 2 * self.rotate();
                        let immediate = self.immediate();
                        ror(immediate, rotate, &mut carry, false)
                    }
                };

                match is_spsr {
                    false => {
                        // User mode can only change flags
                        if cpu.cpsr().mode() == CpuMode::User {
                            mask &= 0xFF000000;
                        }

                        // Make sure operand has the 4th bit
                        if mask & 0xFF != 0 {
                            operand |= 0x10;
                        }

                        let bits = (cpu.cpsr().into_bits() & !mask) | (operand & mask);
                        cpu.set_cpsr(ProgramStatusRegister::from_bits(bits));
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

    pub fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let is_spsr = self.is_spsr();
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

        match self.value.bits(16..=21) as u8 == 0xF {
            true => {
                let rd = self.rd() as usize;
                format!("MRS{} {},{}", cond, rd, psr)
            }
            false => {
                let operand = match self.is_immediate() {
                    false => format!("{}", self.rm()),
                    true => {
                        let rotate = 2 * self.rotate();
                        let immediate = self.immediate();
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

    #[inline]
    pub fn rd(&self) -> Register {
        self.value.bits(12..=15).into()
    }

    #[inline]
    pub fn rm(&self) -> Register {
        self.value.bits(0..=3).into()
    }

    #[inline]
    pub fn is_immediate(&self) -> bool {
        self.value.bit(25)
    }

    #[inline]
    pub fn rotate(&self) -> u32 {
        self.value.bits(8..=11)
    }

    #[inline]
    pub fn immediate(&self) -> u32 {
        self.value.bits(0..=7)
    }

    #[inline]
    pub fn is_spsr(&self) -> bool {
        self.value.bit(22)
    }
}
