use tracing::debug;

use ironboyadvance_arm7tdmi::{
    Condition, CpuAction, CpuState, Dissasemble, ExecutionStrategy,
    cpu::{Arm7tdmiCpu, LastInstruction},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{
    arm::{ArmInstructionFactory, generate_arm_lut},
    thumb::{ThumbInstructionFactory, generate_thumb_lut},
};

pub mod arm;
pub mod thumb;

pub trait Execute {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction;
}

pub struct Interpreter {
    arm_lut: [ArmInstructionFactory; 4096],
    thumb_lut: [ThumbInstructionFactory; 1024],
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            arm_lut: generate_arm_lut(),
            thumb_lut: generate_thumb_lut(),
        }
    }
}

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
                let instruction = (self.arm_lut[lut_index as usize])(instruction);
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
                let instruction = (self.thumb_lut[lut_index as usize])(instruction as u16);
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
