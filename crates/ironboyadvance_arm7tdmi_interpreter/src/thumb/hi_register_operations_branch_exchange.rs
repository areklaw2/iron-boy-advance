use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction, CpuState, HiRegOpsBxOpcode,
    alu::*,
    cpu::{Arm7tdmiCpu, PC},
    memory::MemoryInterface,
    thumb::HiRegisterOperationsBranchExchange,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for HiRegisterOperationsBranchExchange {
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
}
