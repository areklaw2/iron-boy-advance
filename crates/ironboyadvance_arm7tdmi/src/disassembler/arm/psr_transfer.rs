use crate::{CpuMode, Dissasemble, arm::PsrTransfer, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for PsrTransfer {
    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
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

        match self.is_mrs() {
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
}
