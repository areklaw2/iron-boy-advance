use crate::{
    BitOps, CpuAction, CpuState, HiRegOpsBxOpcode, HiRegister, LoRegister,
    alu::{add, cmp, mov},
    cpu::{Arm7tdmiCpu, Instruction, PC},
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct HiRegisterOperationsBranchExchange {
    value: u16,
}

thumb_instruction!(HiRegisterOperationsBranchExchange);

impl Instruction for HiRegisterOperationsBranchExchange {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        use HiRegOpsBxOpcode::*;
        let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
        let destination = match self.h1() {
            true => self.hd() as usize + 8,
            false => self.rd() as usize,
        };
        let operand1 = cpu.register(destination);

        let source = match self.h2() {
            true => self.hs() as usize + 8,
            false => self.rs() as usize,
        };
        let mut operand2 = cpu.register(source);
        if source == PC {
            operand2 &= !0x1
        }

        match self.opcode().into() {
            CMP => {
                cmp(cpu, true, operand1, operand2);
            }
            ADD => {
                let result = add(cpu, false, operand1, operand2);
                cpu.set_register(destination, result);
                if destination == PC {
                    cpu.set_pc(cpu.pc() & !0x1);
                    cpu.pipeline_flush();
                    action = CpuAction::PipelineFlush;
                }
            }
            MOV => {
                let result = mov(cpu, false, operand2, cpu.cpsr().carry());
                cpu.set_register(destination, result);
                if destination == PC {
                    cpu.set_pc(cpu.pc() & !0x1);
                    cpu.pipeline_flush();
                    action = CpuAction::PipelineFlush;
                }
            }
            BX => {
                cpu.cpsr_mut().set_state(CpuState::from_bits((operand2 & 0x1) as u8));
                cpu.set_pc(operand2 & !0x1);
                cpu.pipeline_flush();
                action = CpuAction::PipelineFlush;
            }
        };

        action
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let destination = match self.h1() {
            true => self.hd().to_string(),
            false => self.rd().to_string(),
        };

        let source = match self.h2() {
            true => self.hs().to_string(),
            false => self.rs().to_string(),
        };

        let opcode = HiRegOpsBxOpcode::from(self.opcode());
        format!("{} {},{}", opcode, destination, source)
    }
}

impl HiRegisterOperationsBranchExchange {
    #[inline]
    pub fn rd(&self) -> LoRegister {
        self.value.bits(0..=2).into()
    }

    #[inline]
    pub fn hd(&self) -> HiRegister {
        self.value.bits(0..=2).into()
    }

    #[inline]
    pub fn rs(&self) -> LoRegister {
        self.value.bits(3..=5).into()
    }

    #[inline]
    pub fn hs(&self) -> HiRegister {
        self.value.bits(3..=5).into()
    }

    #[inline]
    pub fn h2(&self) -> bool {
        self.value.bit(6)
    }

    #[inline]
    pub fn h1(&self) -> bool {
        self.value.bit(7)
    }

    #[inline]
    pub fn opcode(&self) -> u16 {
        self.value.bits(8..=9)
    }
}
