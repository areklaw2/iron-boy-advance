use ironboyadvance_arm7tdmi::{alu, cpu::Arm7tdmiCpu, memory::MemoryInterface};

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
