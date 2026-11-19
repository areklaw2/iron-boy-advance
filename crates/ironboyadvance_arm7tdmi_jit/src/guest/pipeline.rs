use ironboyadvance_arm7tdmi::{alu, cpu::Arm7tdmiCpu, memory::MemoryInterface};

use crate::JitInterface;

pub unsafe extern "C" fn pipeline_flush<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.pipeline_flush();
}

pub unsafe extern "C" fn idle_cycle<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.idle_cycle();
}

pub unsafe extern "C" fn multiplier_array_cycles<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, operand: u32) {
    let cpu = unsafe { &mut *cpu };
    let multiplier_cycles = alu::multiplier_array_cycles(operand);
    for _ in 0..multiplier_cycles {
        cpu.idle_cycle();
    }
}

pub unsafe extern "C" fn fetch_next_arm<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, access_pattern: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.advance_pc_arm();

    let pc = cpu.pc() & !0x1;
    cpu.cpu_context_mut().pc = pc;

    let pipeline = *cpu.pipeline();
    cpu.set_next_memory_access(access_pattern as u8);
    let next = cpu.load_32(pc, access_pattern as u8);

    cpu.set_pipeline([pipeline[1], next]);
    let pipeline = *cpu.pipeline();
    cpu.cpu_context_mut().pipeline = pipeline;
}

pub unsafe extern "C" fn fetch_next_thumb<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, access_pattern: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.advance_pc_thumb();

    let pc = cpu.pc() & !0x1;
    cpu.cpu_context_mut().pc = pc;

    let pipeline = *cpu.pipeline();
    cpu.set_next_memory_access(access_pattern as u8);
    let next = cpu.load_16(pc, access_pattern as u8);

    cpu.set_pipeline([pipeline[1], next]);
    let pipeline = *cpu.pipeline();
    cpu.cpu_context_mut().pipeline = pipeline;
}

pub unsafe extern "C" fn should_exit_block<I: MemoryInterface + JitInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &mut *cpu };
    let irq_enabled = !cpu.cpsr().irq_disable();
    cpu.bus_mut().block_exit_pending(irq_enabled) as u32
}
