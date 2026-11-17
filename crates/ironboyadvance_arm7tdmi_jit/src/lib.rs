use std::collections::HashMap;

use dynasmrt::{Assembler, AssemblyOffset, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{
    CpuAction, CpuState, Dissasemble, ExecutionStrategy,
    cpu::{Arm7tdmiCpu, LastInstruction},
    memory::MemoryInterface,
};
use thiserror::Error;
use tracing::debug;

use crate::{arm::compile_arm_block, thumb::compile_thumb_block};
pub use crate::{arm::decode_arm, thumb::decode_thumb};

mod arm;
mod guest;
mod thumb;

#[derive(Error, Debug)]
pub enum JitError {
    #[error("failed to allocate JIT executable memory: {0}")]
    Alloc(#[from] std::io::Error),
}

const PIPELINE_FLUSH: u32 = 0xFFFF_FFFF;
const MAX_BLOCK_INSTRUCTIONS: usize = 32;

pub trait Compile {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>);
    fn ends_block(&self) -> bool;
}

pub struct JitCompiler {
    assembler: Assembler<Aarch64Relocation>,
    blocks: HashMap<(u32, CpuState), AssemblyOffset>,
}

impl JitCompiler {
    pub fn new() -> Result<Self, JitError> {
        let assembler = Assembler::<Aarch64Relocation>::new()?;
        let blocks = HashMap::new();
        Ok(Self { assembler, blocks })
    }

    fn run<I: MemoryInterface>(&mut self, cpu: &mut Arm7tdmiCpu<I>, offset: AssemblyOffset) -> CpuAction {
        self.assembler.commit().unwrap();
        let reader = self.assembler.reader();
        let buffer = reader.lock();
        let block: extern "C" fn(*mut Arm7tdmiCpu<I>) -> u32 = unsafe { std::mem::transmute(buffer.ptr(offset)) };
        match block(cpu as *mut _) {
            PIPELINE_FLUSH => CpuAction::PipelineFlush,
            access => CpuAction::Advance(access as u8),
        }
    }
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new().expect("failed to allocate JIT executable memory")
    }
}

impl ExecutionStrategy for JitCompiler {
    fn cycle<I: MemoryInterface>(&mut self, cpu: &mut Arm7tdmiCpu<I>) {
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

                let instruction = decode_arm(instruction);
                cpu.set_last_instruction(Some(LastInstruction::Arm(instruction)));

                if *cpu.show_logs() {
                    debug!("{}", instruction.disassemble(cpu));
                }

                let address = pc.wrapping_sub(8);
                let offset = *self
                    .blocks
                    .entry((address, cpu_state))
                    .or_insert_with(|| compile_arm_block::<I>(&mut self.assembler, cpu, address, instruction));

                match self.run(cpu, offset) {
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

                let instruction = decode_thumb(instruction as u16);
                cpu.set_last_instruction(Some(LastInstruction::Thumb(instruction)));

                if *cpu.show_logs() {
                    debug!("{}", instruction.disassemble(cpu));
                }

                let address = pc.wrapping_sub(4);
                let offset = *self
                    .blocks
                    .entry((address, cpu_state))
                    .or_insert_with(|| compile_thumb_block(&mut self.assembler, cpu, address, instruction));

                match self.run(cpu, offset) {
                    CpuAction::Advance(memory_access) => {
                        cpu.advance_pc_thumb();
                        cpu.set_next_memory_access(memory_access);
                    }
                    CpuAction::PipelineFlush => {}
                };
            }
        }
    }

    fn flush_cache(&mut self) {
        self.blocks.clear();
    }
}

fn emit_prologue(assembler: &mut Assembler<Aarch64Relocation>) {
    dynasm! { assembler
        ; .arch aarch64
        ; stp x24, x23, [sp, #-64]!
        ; stp x22, x21, [sp, #16]
        ; stp x20, x19, [sp, #32]
        ; str x30, [sp, #48]
        ; mov x19, x0
    }
}

fn emit_epilogue(assembler: &mut Assembler<Aarch64Relocation>) {
    dynasm! { assembler
        ; .arch aarch64
        ; ldr x30, [sp, #48]
        ; ldp x20, x19, [sp, #32]
        ; ldp x22, x21, [sp, #16]
        ; ldp x24, x23, [sp], #64
        ; ret
    }
}

fn emit_call(assembler: &mut Assembler<Aarch64Relocation>, target: *const ()) {
    let address = target as usize;

    let byte_0 = (address & 0xFFFF) as u32;
    let byte_1 = ((address >> 16) & 0xFFFF) as u32;
    let byte_2 = ((address >> 32) & 0xFFFF) as u32;
    let byte_3 = ((address >> 48) & 0xFFFF) as u32;
    // move address into x9 and then jump
    dynasm! { assembler
        ; .arch aarch64
        ; movz x9, #byte_0
        ; movk x9, #byte_1, lsl #16
        ; movk x9, #byte_2, lsl #32
        ; movk x9, #byte_3, lsl #48
        ; blr x9 // jump to the address
    }
}

fn emit_action(assembler: &mut Assembler<Aarch64Relocation>, action: CpuAction) {
    match action {
        CpuAction::PipelineFlush => dynasm! { assembler
            ; .arch aarch64
            ; movn w0, #0
        },
        CpuAction::Advance(access) => dynasm! { assembler
            ; .arch aarch64
            ; mov w0, #access as u32
        },
    }
}

#[allow(clippy::useless_conversion)]
fn emit_immediate_32(assembler: &mut Assembler<Aarch64Relocation>, register: u8, value: u32) {
    let low = value & 0xFFFF;
    let high = (value >> 16) & 0xFFFF;
    dynasm! { assembler
        ; .arch aarch64
        ; movz W(register), #low
        ; movk W(register), #high, lsl #16
    }
}
