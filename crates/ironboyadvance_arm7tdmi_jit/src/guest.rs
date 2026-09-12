use ironboyadvance_arm7tdmi::{
    CpuState, Exception, alu, barrel_shifter, cpu::Arm7tdmiCpu, memory::MemoryInterface, psr::ProgramStatusRegister,
};
use ironboyadvance_common::memory::MemoryAccess;

use crate::PIPELINE_FLUSH;

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

pub unsafe extern "C" fn cpsr_carry<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.cpsr().carry() as u32
}

pub unsafe extern "C" fn set_cpsr_state<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
}

pub unsafe extern "C" fn set_cpsr<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.set_cpsr(ProgramStatusRegister::from_bits_with_defaults(value));
}

pub unsafe extern "C" fn set_cpsr_to_spsr<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.set_cpsr(cpu.spsr());
}

pub unsafe extern "C" fn pipeline_flush<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.pipeline_flush();
}

pub unsafe extern "C" fn idle_cycle<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.idle_cycle();
}

pub unsafe extern "C" fn undefined_exception<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) {
    let cpu = unsafe { &mut *cpu };
    cpu.exception(Exception::Undefined);
}

pub unsafe extern "C" fn software_interrupt<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, function: u32) -> u32 {
    let cpu = unsafe { &mut *cpu };
    match !cpu.bios_loaded() && cpu.bios_call(function) {
        true => (MemoryAccess::Instruction | MemoryAccess::Sequential) as u32,
        false => {
            cpu.exception(Exception::SoftwareInterrupt);
            PIPELINE_FLUSH
        }
    }
}

pub unsafe extern "C" fn and<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
    carry: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::and(cpu, set_flags != 0, operand1, operand2, carry != 0)
}

pub unsafe extern "C" fn eor<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
    carry: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::eor(cpu, set_flags != 0, operand1, operand2, carry != 0)
}

pub unsafe extern "C" fn sub<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::sub(cpu, set_flags != 0, operand1, operand2)
}

pub unsafe extern "C" fn rsb<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::rsb(cpu, set_flags != 0, operand1, operand2)
}

pub unsafe extern "C" fn add<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::add(cpu, set_flags != 0, operand1, operand2)
}

pub unsafe extern "C" fn adc<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::adc(cpu, set_flags != 0, operand1, operand2)
}

pub unsafe extern "C" fn sbc<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::sbc(cpu, set_flags != 0, operand1, operand2)
}

pub unsafe extern "C" fn rsc<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::rsc(cpu, set_flags != 0, operand1, operand2)
}

pub unsafe extern "C" fn tst<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
    carry: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::tst(cpu, set_flags != 0, operand1, operand2, carry != 0)
}

pub unsafe extern "C" fn teq<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
    carry: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::teq(cpu, set_flags != 0, operand1, operand2, carry != 0)
}

pub unsafe extern "C" fn cmp<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::cmp(cpu, set_flags != 0, operand1, operand2)
}

pub unsafe extern "C" fn cmn<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::cmn(cpu, set_flags != 0, operand1, operand2)
}

pub unsafe extern "C" fn orr<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
    carry: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::orr(cpu, set_flags != 0, operand1, operand2, carry != 0)
}

pub unsafe extern "C" fn mov<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand2: u32,
    carry: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::mov(cpu, set_flags != 0, operand2, carry != 0)
}

pub unsafe extern "C" fn bic<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand1: u32,
    operand2: u32,
    carry: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::bic(cpu, set_flags != 0, operand1, operand2, carry != 0)
}

pub unsafe extern "C" fn mvn<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    set_flags: u32,
    operand2: u32,
    carry: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    alu::mvn(cpu, set_flags != 0, operand2, carry != 0)
}

#[repr(C)]
pub struct ShiftResult {
    pub value: u32,
    pub carry: u32,
}

pub extern "C" fn lsl_by_register(value: u32, amount: u32, carry: u32) -> ShiftResult {
    let mut carry = carry != 0;
    let value = barrel_shifter::lsl(value, amount, &mut carry);
    ShiftResult {
        value,
        carry: carry as u32,
    }
}

pub extern "C" fn lsr_by_register(value: u32, amount: u32, carry: u32) -> ShiftResult {
    let mut carry = carry != 0;
    let value = barrel_shifter::lsr(value, amount, &mut carry, false);
    ShiftResult {
        value,
        carry: carry as u32,
    }
}

pub extern "C" fn asr_by_register(value: u32, amount: u32, carry: u32) -> ShiftResult {
    let mut carry = carry != 0;
    let value = barrel_shifter::asr(value, amount, &mut carry, false);
    ShiftResult {
        value,
        carry: carry as u32,
    }
}

pub extern "C" fn ror_by_register(value: u32, amount: u32, carry: u32) -> ShiftResult {
    let mut carry = carry != 0;
    let value = barrel_shifter::ror(value, amount, &mut carry, false);
    ShiftResult {
        value,
        carry: carry as u32,
    }
}
