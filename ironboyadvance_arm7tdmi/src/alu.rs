use crate::{cpu::Arm7tdmiCpu, memory::MemoryInterface};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataProcessingAluOpcode {
    AND,
    EOR,
    SUB,
    RSB,
    ADD,
    ADC,
    SBC,
    RSC,
    TST,
    TEQ,
    CMP,
    CMN,
    ORR,
    MOV,
    BIC,
    MVN,
}

impl From<u32> for DataProcessingAluOpcode {
    fn from(value: u32) -> Self {
        use DataProcessingAluOpcode::*;
        match value {
            0b0000 => AND,
            0b0001 => EOR,
            0b0010 => SUB,
            0b0011 => RSB,
            0b0100 => ADD,
            0b0101 => ADC,
            0b0110 => SBC,
            0b0111 => RSC,
            0b1000 => TST,
            0b1001 => TEQ,
            0b1010 => CMP,
            0b1011 => CMN,
            0b1100 => ORR,
            0b1101 => MOV,
            0b1110 => BIC,
            0b1111 => MVN,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MovCmpAddSubImmdiateOpcode {
    MOV,
    CMP,
    ADD,
    SUB,
}

impl From<u16> for MovCmpAddSubImmdiateOpcode {
    fn from(value: u16) -> Self {
        use MovCmpAddSubImmdiateOpcode::*;
        match value {
            0b00 => MOV,
            0b01 => CMP,
            0b10 => ADD,
            0b11 => SUB,
            _ => unreachable!(),
        }
    }
}

pub fn and<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32, carry: bool) -> u32 {
    let result = operand1 & operand2;
    if set_flags {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(carry);
    }
    result
}

pub fn eor<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32, carry: bool) -> u32 {
    let result = operand1 ^ operand2;
    if set_flags {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(carry);
    }
    result
}

pub fn sub<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32) -> u32 {
    let result = operand1.wrapping_sub(operand2);
    if set_flags {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(operand1 as u64 >= operand2 as u64);
        cpu.set_overflow(((operand1 ^ operand2) & (operand1 ^ result)) >> 31 != 0);
    }
    result
}

pub fn rsb<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32) -> u32 {
    sub(cpu, set_flags, operand1, operand2)
}

pub fn add<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32) -> u32 {
    let result = operand1.wrapping_add(operand2);
    if set_flags {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(result < operand1);
        cpu.set_overflow((!(operand1 ^ operand2) & (operand1 ^ result)) >> 31 != 0);
    }
    result
}

pub fn adc<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32) -> u32 {
    let result = operand1 as u64 + operand2 as u64 + cpu.cpsr().carry() as u64;
    if set_flags {
        cpu.set_negative((result >> 31) & 0b1 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(result >> 32 != 0);
        cpu.set_overflow((!(operand1 ^ operand2) & (operand1 ^ result as u32)) >> 31 != 0);
    }
    result as u32
}

pub fn sbc<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32) -> u32 {
    let operand3 = cpu.cpsr().carry() as u32 ^ 1;
    let result = operand1.wrapping_sub(operand2).wrapping_sub(operand3);
    if set_flags {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(operand1 as u64 >= operand2 as u64 + operand3 as u64);
        cpu.set_overflow(((operand1 ^ operand2) & (operand1 ^ result)) >> 31 != 0);
    }
    result
}

pub fn rsc<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32) -> u32 {
    sbc(cpu, set_flags, operand1, operand2)
}

pub fn tst<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32, carry: bool) -> u32 {
    and(cpu, set_flags, operand1, operand2, carry)
}

pub fn teq<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32, carry: bool) -> u32 {
    eor(cpu, set_flags, operand1, operand2, carry)
}

pub fn cmp<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32) -> u32 {
    sub(cpu, set_flags, operand1, operand2)
}

pub fn cmn<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32) -> u32 {
    add(cpu, set_flags, operand1, operand2)
}

pub fn orr<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32, carry: bool) -> u32 {
    let result = operand1 | operand2;
    if set_flags {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(carry);
    }
    result
}

pub fn bic<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand1: u32, operand2: u32, carry: bool) -> u32 {
    let result = operand1 & !operand2;
    if set_flags {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(carry);
    }
    result
}

pub fn mov<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand2: u32, carry: bool) -> u32 {
    let result = operand2;
    if set_flags {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(carry);
    }
    result
}

pub fn mvn<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, set_flags: bool, operand2: u32, carry: bool) -> u32 {
    let result = !operand2;
    if set_flags {
        cpu.set_negative(result >> 31 != 0);
        cpu.set_zero(result == 0);
        cpu.set_carry(carry);
    }
    result
}

pub fn multiplier_array_cycles(multiplier: u32) -> usize {
    if multiplier & 0xFF == multiplier {
        1
    } else if multiplier & 0xFFFF == multiplier {
        2
    } else if multiplier & 0xFFFFFF == multiplier {
        3
    } else {
        4
    }
}
