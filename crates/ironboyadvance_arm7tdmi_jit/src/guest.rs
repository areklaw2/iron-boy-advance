use ironboyadvance_arm7tdmi::{CpuState, Exception, cpu::Arm7tdmiCpu, memory::MemoryInterface};

pub unsafe extern "C" fn pc<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.pc()
}

pub unsafe extern "C" fn set_pc<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.set_pc(value);
}

pub unsafe extern "C" fn register<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, register: u32) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.register(register as usize)
}

pub unsafe extern "C" fn set_register<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, register: u32, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.set_register(register as usize, value);
}

pub unsafe extern "C" fn set_cpsr_state<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
}

pub unsafe extern "C" fn pipeline_flush<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.pipeline_flush();
}

pub unsafe extern "C" fn undefined_exception<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.exception(Exception::Undefined);
}

pub unsafe extern "C" fn software_interrupt_exception<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.exception(Exception::SoftwareInterrupt);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn bios_loaded<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> bool {
    let cpu = unsafe { &*cpu };
    cpu.bios_loaded()
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn bios_call<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, function: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.bios_call(function);
}
