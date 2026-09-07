use dynasmrt::{Assembler, AssemblyOffset, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{
    Condition, CpuAction, CpuState, Dissasemble, Exception, ExecutionStrategy,
    cpu::{Arm7tdmiCpu, LastInstruction},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;
use thiserror::Error;
use tracing::debug;

use crate::{arm::decode_arm, thumb::decode_thumb};

pub mod arm;
pub mod thumb;

#[derive(Error, Debug)]
pub enum JitError {
    #[error("failed to allocate JIT executable memory: {0}")]
    Alloc(#[from] std::io::Error),
}

pub trait Compile {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>);
}

pub struct JitCompiler {
    assembler: Assembler<Aarch64Relocation>,
}

impl JitCompiler {
    pub fn new() -> Result<Self, JitError> {
        let assembler = Assembler::<Aarch64Relocation>::new()?;
        Ok(Self { assembler })
    }

    pub fn run<I: MemoryInterface>(&mut self, cpu: &mut Arm7tdmiCpu<I>, offset: AssemblyOffset) -> CpuAction {
        self.assembler.commit().unwrap();
        let reader = self.assembler.reader();
        let buffer = reader.lock();
        let block: extern "C" fn(*mut Arm7tdmiCpu<I>) -> u32 = unsafe { std::mem::transmute(buffer.ptr(offset)) };
        match block(cpu as *mut _) {
            0xFFFF_FFFF => CpuAction::PipelineFlush,
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

                let condition = instruction.cond();
                if condition != Condition::AL && !cpu.is_condition_met(condition) {
                    cpu.advance_pc_arm();
                    cpu.set_next_memory_access(MemoryAccess::Instruction | MemoryAccess::Sequential);
                    return;
                }

                let offset = self.assembler.offset();
                instruction.compile::<I>(&mut self.assembler);
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

                let offset = self.assembler.offset();
                instruction.compile::<I>(&mut self.assembler);
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
}

fn emit_prologue(assembler: &mut Assembler<Aarch64Relocation>) {
    dynasm! { assembler
        ; .arch aarch64
        ; stp x20, x19, [sp, #-32]!
        ; str x30, [sp, #16]
    }
}

fn emit_epilogue(assembler: &mut Assembler<Aarch64Relocation>) {
    dynasm! { assembler
        ; .arch aarch64
        ; ldr x30, [sp, #16]
        ; ldp x20, x19, [sp], #32
        ; ret
    }
}

fn emit_prologue_link_only(assembler: &mut Assembler<Aarch64Relocation>) {
    dynasm! { assembler
        ; .arch aarch64
        ; str x30, [sp, #-16]!
    }
}

fn emit_epilogue_link_only(assembler: &mut Assembler<Aarch64Relocation>) {
    dynasm! { assembler
        ; .arch aarch64
        ; ldr x30, [sp], #16
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

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn trampoline_pc<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.pc()
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn trampoline_set_pc<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.set_pc(value);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn trampoline_register<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, register: u32) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.register(register as usize)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn trampoline_set_register<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, register: u32, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.set_register(register as usize, value);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn trampoline_set_cpsr_state<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn trampoline_pipeline_flush<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.pipeline_flush();
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn trampoline_undefined_exception<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.exception(Exception::Undefined);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn trampoline_software_interrupt_exception<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.exception(Exception::SoftwareInterrupt);
}
