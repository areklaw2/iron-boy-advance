use ironboyadvance_arm7tdmi::{CpuMode, CpuState, cpu::Arm7tdmiCpu, memory::MemoryInterface, psr::ProgramStatusRegister};

pub unsafe extern "C" fn is_condition_met<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, condition: u32) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.is_condition_met(condition.into()) as u32
}

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

pub unsafe extern "C" fn spsr<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.spsr().into_bits()
}

pub unsafe extern "C" fn set_spsr<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.set_spsr(ProgramStatusRegister::from_bits_with_defaults(value));
}

pub unsafe extern "C" fn cpsr<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.cpsr().into_bits()
}

pub unsafe extern "C" fn cpsr_carry<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.cpsr().carry() as u32
}

pub unsafe extern "C" fn set_cpsr_negative<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.cpsr_mut().set_negative(value >> 31 != 0);
}

pub unsafe extern "C" fn set_cpsr_zero<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.cpsr_mut().set_zero(value == 0);
}

pub unsafe extern "C" fn set_cpsr_carry<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.cpsr_mut().set_carry(value != 0);
}

pub unsafe extern "C" fn cpsr_mode<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.cpsr().mode() as u32
}

pub unsafe extern "C" fn set_cpsr_mode<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.cpsr_mut().set_mode(CpuMode::from_bits(value as u8));
}

pub unsafe extern "C" fn set_cpsr<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.set_cpsr(ProgramStatusRegister::from_bits_with_defaults(value));
}

pub unsafe extern "C" fn set_cpsr_to_spsr<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.set_cpsr(cpu.spsr());
}

pub unsafe extern "C" fn set_cpsr_state<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
}

pub unsafe extern "C" fn enter_user_mode<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &mut *cpu };
    let mode = cpu.cpsr().mode();
    if ![CpuMode::User, CpuMode::System].contains(&mode) {
        cpu.cpsr_mut().set_mode(CpuMode::User);
    }
    mode as u32
}
