use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction, CpuMode, arm::PsrTransfer, cpu::Arm7tdmiCpu, memory::MemoryInterface, psr::ProgramStatusRegister,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for PsrTransfer {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let is_spsr = self.is_spsr();
        match self.is_mrs() {
            true => {
                let rd = self.rd() as usize;
                let psr = match is_spsr {
                    false => *cpu.cpsr(),
                    true => cpu.spsr(),
                };
                cpu.set_register(rd, psr.into_bits());
            }
            false => {
                let mask = self.psr_mask();

                let mut operand = match self.is_immediate() {
                    false => cpu.register(self.rm() as usize),
                    true => {
                        let rotate = 2 * self.rotate();
                        let immediate = self.immediate();
                        immediate.rotate_right(rotate)
                    }
                };

                match is_spsr {
                    false => {
                        if cpu.cpsr().mode() == CpuMode::User {
                            let mask = mask & 0xFF000000;
                            let bits = (cpu.cpsr().into_bits() & !mask) | (operand & mask);
                            cpu.set_cpsr(ProgramStatusRegister::from_bits_with_defaults(bits));
                        } else {
                            if mask & 0xFF != 0 {
                                operand |= 0x10;
                            }
                            let bits = (cpu.cpsr().into_bits() & !mask) | (operand & mask);
                            cpu.set_cpsr(ProgramStatusRegister::from_bits_with_defaults(bits));
                        }
                    }
                    true => {
                        if cpu.cpsr().mode() != CpuMode::User && cpu.cpsr().mode() != CpuMode::System {
                            let bits = (cpu.spsr().into_bits() & !mask) | (operand & mask);
                            cpu.set_spsr(ProgramStatusRegister::from_bits_with_defaults(bits));
                        }
                    }
                }
            }
        }
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}
