use ironboyadvance_arm7tdmi::{Exception, cpu::Arm7tdmiCpu, memory::MemoryInterface};
use ironboyadvance_common::memory::MemoryAccess;

use crate::PIPELINE_FLUSH;

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

pub unsafe extern "C" fn should_exit_block<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>) -> u32 {
    let cpu = unsafe { &mut *cpu };
    let exit_block = cpu.halted() || (!cpu.cpsr().irq_disable() && cpu.interrupt_pending());
    exit_block as u32
}
