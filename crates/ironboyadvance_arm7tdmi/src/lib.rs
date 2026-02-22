mod alu;
mod arm;
mod barrel_shifter;
pub mod cpu;
mod disassembler;
pub mod memory;
mod psr;
mod test;
mod thumb;

pub const CPU_CLOCK_SPEED: u32 = 16777216;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum CpuAction {
    Advance(u8),
    PipelineFlush,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum CpuMode {
    User = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
    Invalid,
}

impl CpuMode {
    pub const fn from_bits(bits: u8) -> Self {
        use CpuMode::*;
        match bits {
            0b10000 => User,
            0b10001 => Fiq,
            0b10010 => Irq,
            0b10011 => Supervisor,
            0b10111 => Abort,
            0b11011 => Undefined,
            0b11111 => System,
            _ => Invalid,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum CpuState {
    Arm = 0,
    Thumb = 1,
}

impl CpuState {
    pub const fn from_bits(bits: u8) -> Self {
        use CpuState::*;
        match bits {
            0 => Arm,
            1 => Thumb,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        use Register::*;
        match value {
            0b0000 => R0,
            0b0001 => R1,
            0b0010 => R2,
            0b0011 => R3,
            0b0100 => R4,
            0b0101 => R5,
            0b0110 => R6,
            0b0111 => R7,
            0b1000 => R8,
            0b1001 => R9,
            0b1010 => R10,
            0b1011 => R11,
            0b1100 => R12,
            0b1101 => R13,
            0b1110 => R14,
            _ => R15,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum LoRegister {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

impl From<u16> for LoRegister {
    fn from(value: u16) -> Self {
        use LoRegister::*;
        match value {
            0b000 => R0,
            0b001 => R1,
            0b010 => R2,
            0b011 => R3,
            0b100 => R4,
            0b101 => R5,
            0b110 => R6,
            _ => R7,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum HiRegister {
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl From<u16> for HiRegister {
    fn from(value: u16) -> Self {
        use HiRegister::*;
        match value {
            0b000 => R8,
            0b001 => R9,
            0b010 => R10,
            0b011 => R11,
            0b100 => R12,
            0b101 => R13,
            0b110 => R14,
            _ => R15,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Condition {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    AL,
}

impl From<u32> for Condition {
    fn from(value: u32) -> Self {
        use Condition::*;
        match value {
            0b0000 => EQ,
            0b0001 => NE,
            0b0010 => CS,
            0b0011 => CC,
            0b0100 => MI,
            0b0101 => PL,
            0b0110 => VS,
            0b0111 => VC,
            0b1000 => HI,
            0b1001 => LS,
            0b1010 => GE,
            0b1011 => LT,
            0b1100 => GT,
            0b1101 => LE,
            0b1110 => AL,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum DataProcessingOpcode {
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

impl From<u32> for DataProcessingOpcode {
    fn from(value: u32) -> Self {
        use DataProcessingOpcode::*;
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

// THUMB
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum MovCmpAddSubImmediateOpcode {
    MOV,
    CMP,
    ADD,
    SUB,
}

impl From<u16> for MovCmpAddSubImmediateOpcode {
    fn from(value: u16) -> Self {
        use MovCmpAddSubImmediateOpcode::*;
        match value {
            0b00 => MOV,
            0b01 => CMP,
            0b10 => ADD,
            0b11 => SUB,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum AluOperationsOpcode {
    AND,
    EOR,
    LSL,
    LSR,
    ASR,
    ADC,
    SBC,
    ROR,
    TST,
    NEG,
    CMP,
    CMN,
    ORR,
    MUL,
    BIC,
    MVN,
}

impl From<u16> for AluOperationsOpcode {
    fn from(value: u16) -> Self {
        use AluOperationsOpcode::*;
        match value {
            0b0000 => AND,
            0b0001 => EOR,
            0b0010 => LSL,
            0b0011 => LSR,
            0b0100 => ASR,
            0b0101 => ADC,
            0b0110 => SBC,
            0b0111 => ROR,
            0b1000 => TST,
            0b1001 => NEG,
            0b1010 => CMP,
            0b1011 => CMN,
            0b1100 => ORR,
            0b1101 => MUL,
            0b1110 => BIC,
            0b1111 => MVN,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum HiRegOpsBxOpcode {
    ADD,
    CMP,
    MOV,
    BX,
}

impl From<u16> for HiRegOpsBxOpcode {
    fn from(value: u16) -> Self {
        use HiRegOpsBxOpcode::*;
        match value {
            0b00 => ADD,
            0b01 => CMP,
            0b10 => MOV,
            0b11 => BX,
            _ => unreachable!(),
        }
    }
}

#[allow(dead_code)]
pub(crate) enum Exception {
    Reset = 0x00,
    Undefined = 0x04,
    SoftwareInterrupt = 0x08,
    // AbortPrefetch = 0x0C,
    // AbortData = 0x10,
    // Reserved = 0x14,
    Irq = 0x18,
    Fiq = 0x1C,
}

use std::mem::size_of;
use std::ops::RangeInclusive;

pub trait BitOps {
    fn bit(&self, index: usize) -> bool;
    fn set_bit(&mut self, index: usize, value: bool);
    fn bits(&self, range: RangeInclusive<usize>) -> Self;
}

macro_rules! impl_bitops {
    ($($t:ty),+ $(,)?) => {
        $(
            impl BitOps for $t {
                fn bit(&self, index: usize) -> bool {
                    debug_assert!(index < size_of::<$t>() * 8);
                    let mask = 1 << index;
                    (self & mask) != 0
                }

                fn set_bit(&mut self, index: usize, value: bool) {
                    debug_assert!(index < size_of::<$t>() * 8);
                    let mask = 1 << index;
                    if value {
                        *self |= mask;
                    } else {
                        *self &= !mask;
                    }
                }

                fn bits(&self, range: RangeInclusive<usize>) -> Self {
                    let start = *range.start();
                    let end = *range.end();
                    debug_assert!(end < size_of::<$t>() * 8);
                    debug_assert!(start <= end);

                    let length = end - start + 1;
                    let bit_width = size_of::<$t>() * 8;
                    let mask = if length >= bit_width {
                        <$t>::MAX
                    } else {
                        ((1 as $t) << length) - 1
                    };
                    (self >> start) & mask
                }
            }
        )+
    };
}

impl_bitops!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8_bit_operations() {
        let mut value: u8 = 0b10110;

        assert_eq!(value.bit(0), false);
        assert_eq!(value.bit(1), true);
        assert_eq!(value.bit(2), true);
        assert_eq!(value.bit(3), false);
        assert_eq!(value.bit(4), true);
        assert_eq!(value.bit(7), false);

        value.set_bit(0, true);
        assert_eq!(value, 0b10111);
        value.set_bit(4, false);
        assert_eq!(value, 0b00111);
        value.set_bit(7, true);
        assert_eq!(value, 0b10000111);

        assert_eq!(value.bits(0..=2), 0b111);
        assert_eq!(value.bits(7..=7), 0b1);
        assert_eq!(value.bits(0..=7), value);
    }

    #[test]
    fn u16_bit_operations() {
        let mut value: u16 = 0b1010110;

        assert_eq!(value.bit(0), false);
        assert_eq!(value.bit(1), true);
        assert_eq!(value.bit(15), false);

        value.set_bit(0, true);
        assert_eq!(value, 0b1010111);
        value.set_bit(15, true);
        assert_eq!(value, 0b1000000001010111);

        assert_eq!(value.bits(0..=3), 0b0111);
        assert_eq!(value.bits(15..=15), 0b1);
    }

    #[test]
    fn u32_bit_operations() {
        let mut value: u32 = 0b10110;

        assert_eq!(value.bit(0), false);
        assert_eq!(value.bit(1), true);
        assert_eq!(value.bit(31), false);

        value.set_bit(31, true);
        assert_eq!(value.bit(31), true);

        assert_eq!(value.bits(1..=3), 0b011);
    }

    #[test]
    fn u64_bit_operations() {
        let mut value: u64 = 0b10110;

        assert_eq!(value.bit(0), false);
        assert_eq!(value.bit(1), true);
        assert_eq!(value.bit(63), false);

        value.set_bit(63, true);
        assert_eq!(value.bit(63), true);

        assert_eq!(value.bits(1..=4), 0b1011);
    }

    #[test]
    fn set_bit_clear() {
        let mut value: u32 = 0b1111;

        value.set_bit(0, false);
        assert_eq!(value, 0b1110);

        value.set_bit(1, false);
        assert_eq!(value, 0b1100);

        value.set_bit(2, false);
        value.set_bit(3, false);
        assert_eq!(value, 0);
    }

    #[test]
    fn set_and_get_bit() {
        let mut value: u16 = 0;

        for i in 0..16 {
            value.set_bit(i, true);
            assert_eq!(value.bit(i), true);
        }

        for i in 0..16 {
            value.set_bit(i, false);
            assert_eq!(value.bit(i), false);
        }
    }

    #[test]
    fn bits_single_bit() {
        let value: u8 = 0b10110;
        assert_eq!(value.bits(0..=0), if value.bit(0) { 1 } else { 0 });
        assert_eq!(value.bits(1..=1), if value.bit(1) { 1 } else { 0 });
        assert_eq!(value.bits(4..=4), if value.bit(4) { 1 } else { 0 });
    }

    #[test]
    fn bits_edge_cases() {
        let value: u8 = 0b11001010;
        assert_eq!(value.bits(0..=0), 0);
        assert_eq!(value.bits(7..=7), 1);
        assert_eq!(value.bits(0..=7), value);
        assert_eq!(value.bits(2..=5), 0b0010);
    }
}
