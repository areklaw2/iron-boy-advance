use tracing::debug;

use ironboyadvance_arm7tdmi::{
    Condition, CpuAction, CpuState, ExecutionStrategy,
    cpu::{Arm7tdmiCpu, Instruction, LastInstruction},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;

pub trait Intrepret {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction;
}

pub struct Interpreter;

impl ExecutionStrategy for Interpreter {
    fn cycle<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) {
        let pc = cpu.pc() & !0x1;
        let cpu_state = cpu.cpsr().state();
        let context = cpu.cpu_context_mut();
        context.pc = pc;
        context.cpu_state = cpu_state;

        match cpu_state {
            CpuState::Arm => {
                let pipeline = *cpu.pipeline();
                let instruction = pipeline[0];
                let next_memory_access = *cpu.next_memory_access();
                let next = cpu.load_32(pc, next_memory_access);
                cpu.set_pipeline([pipeline[1], next]);
                let pipeline = *cpu.pipeline();
                cpu.cpu_context_mut().pipeline = pipeline;

                let lut_index = ((instruction >> 16) & 0x0FF0) | ((instruction >> 4) & 0x000F);
                let instruction = (cpu.arm_lut()[lut_index as usize])(instruction);
                cpu.set_last_instruction(Some(LastInstruction::Arm(instruction)));

                if *cpu.show_logs() {
                    debug!("{}", instruction.disassemble(cpu));
                }

                let condition = instruction.cond();
                if condition != Condition::AL && !cpu.is_condition_met(condition) {
                    cpu.advance_pc_arm();
                    cpu.set_next_memory_access(MemoryAccess::Instruction | MemoryAccess::Sequential);
                    return;
                }
                match instruction.execute(cpu) {
                    CpuAction::Advance(memory_access) => {
                        cpu.advance_pc_arm();
                        cpu.set_next_memory_access(memory_access);
                    }
                    CpuAction::PipelineFlush => {}
                };
            }
            CpuState::Thumb => {
                let pipeline = *cpu.pipeline();
                let instruction = pipeline[0];
                let next_memory_access = *cpu.next_memory_access();
                let next = cpu.load_16(pc, next_memory_access);
                cpu.set_pipeline([pipeline[1], next]);
                let pipeline = *cpu.pipeline();
                cpu.cpu_context_mut().pipeline = pipeline;

                let lut_index = instruction as u16 >> 6;
                let instruction = (cpu.thumb_lut()[lut_index as usize])(instruction as u16);
                cpu.set_last_instruction(Some(LastInstruction::Thumb(instruction)));

                if *cpu.show_logs() {
                    debug!("{}", instruction.disassemble(cpu));
                }

                match instruction.execute(cpu) {
                    CpuAction::Advance(memory_access) => {
                        cpu.advance_pc_thumb();
                        cpu.set_next_memory_access(memory_access);
                    }
                    CpuAction::PipelineFlush => {}
                };
            }
        }
    }
}
