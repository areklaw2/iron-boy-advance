mod alu;
mod arm;
mod barrel_shifter;
pub mod bits;
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
        match bits {
            0b10000 => Self::User,
            0b10001 => Self::Fiq,
            0b10010 => Self::Irq,
            0b10011 => Self::Supervisor,
            0b10111 => Self::Abort,
            0b11011 => Self::Undefined,
            0b11111 => Self::System,
            _ => Self::Invalid,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CpuState {
    Arm = 0,
    Thumb = 1,
}

impl CpuState {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0 => Self::Arm,
            1 => Self::Thumb,
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
        match value {
            0b0000 => Self::R0,
            0b0001 => Self::R1,
            0b0010 => Self::R2,
            0b0011 => Self::R3,
            0b0100 => Self::R4,
            0b0101 => Self::R5,
            0b0110 => Self::R6,
            0b0111 => Self::R7,
            0b1000 => Self::R8,
            0b1001 => Self::R9,
            0b1010 => Self::R10,
            0b1011 => Self::R11,
            0b1100 => Self::R12,
            0b1101 => Self::R13,
            0b1110 => Self::R14,
            _ => Self::R15,
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
        match value {
            0b000 => Self::R0,
            0b001 => Self::R1,
            0b010 => Self::R2,
            0b011 => Self::R3,
            0b100 => Self::R4,
            0b101 => Self::R5,
            0b110 => Self::R6,
            _ => Self::R7,
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
        match value {
            0b000 => Self::R8,
            0b001 => Self::R9,
            0b010 => Self::R10,
            0b011 => Self::R11,
            0b100 => Self::R12,
            0b101 => Self::R13,
            0b110 => Self::R14,
            _ => Self::R15,
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
        match value {
            0b0000 => Self::EQ,
            0b0001 => Self::NE,
            0b0010 => Self::CS,
            0b0011 => Self::CC,
            0b0100 => Self::MI,
            0b0101 => Self::PL,
            0b0110 => Self::VS,
            0b0111 => Self::VC,
            0b1000 => Self::HI,
            0b1001 => Self::LS,
            0b1010 => Self::GE,
            0b1011 => Self::LT,
            0b1100 => Self::GT,
            0b1101 => Self::LE,
            0b1110 => Self::AL,
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
        match value {
            0b0000 => Self::AND,
            0b0001 => Self::EOR,
            0b0010 => Self::SUB,
            0b0011 => Self::RSB,
            0b0100 => Self::ADD,
            0b0101 => Self::ADC,
            0b0110 => Self::SBC,
            0b0111 => Self::RSC,
            0b1000 => Self::TST,
            0b1001 => Self::TEQ,
            0b1010 => Self::CMP,
            0b1011 => Self::CMN,
            0b1100 => Self::ORR,
            0b1101 => Self::MOV,
            0b1110 => Self::BIC,
            0b1111 => Self::MVN,
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
        match value {
            0b00 => Self::MOV,
            0b01 => Self::CMP,
            0b10 => Self::ADD,
            0b11 => Self::SUB,
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
        match value {
            0b0000 => Self::AND,
            0b0001 => Self::EOR,
            0b0010 => Self::LSL,
            0b0011 => Self::LSR,
            0b0100 => Self::ASR,
            0b0101 => Self::ADC,
            0b0110 => Self::SBC,
            0b0111 => Self::ROR,
            0b1000 => Self::TST,
            0b1001 => Self::NEG,
            0b1010 => Self::CMP,
            0b1011 => Self::CMN,
            0b1100 => Self::ORR,
            0b1101 => Self::MUL,
            0b1110 => Self::BIC,
            0b1111 => Self::MVN,
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
        match value {
            0b00 => Self::ADD,
            0b01 => Self::CMP,
            0b10 => Self::MOV,
            0b11 => Self::BX,
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

        assert!(!value.bit(0));
        assert!(value.bit(1));
        assert!(value.bit(2));
        assert!(!value.bit(3));
        assert!(value.bit(4));
        assert!(!value.bit(7));

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

        assert!(!value.bit(0));
        assert!(value.bit(1));
        assert!(!value.bit(15));

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

        assert!(!value.bit(0));
        assert!(value.bit(1));
        assert!(!value.bit(31));

        value.set_bit(31, true);
        assert!(value.bit(31));

        assert_eq!(value.bits(1..=3), 0b011);
    }

    #[test]
    fn u64_bit_operations() {
        let mut value: u64 = 0b10110;

        assert!(!value.bit(0));
        assert!(value.bit(1));
        assert!(!value.bit(63));

        value.set_bit(63, true);
        assert!(value.bit(63));

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
            assert!(value.bit(i));
        }

        for i in 0..16 {
            value.set_bit(i, false);
            assert!(!value.bit(i));
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
