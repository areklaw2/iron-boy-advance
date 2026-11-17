use ironboyadvance_arm7tdmi::{alu, cpu::Arm7tdmiCpu, memory::MemoryInterface};

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
