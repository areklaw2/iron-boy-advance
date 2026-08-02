use bitfields::bitflag;
use std::fmt;

mod alu;
mod arm;
mod barrel_shifter;
pub mod cpu;
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

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CpuMode {
    User = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
    #[base]
    Invalid = 0,
}

impl fmt::Display for CpuMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "usr"),
            Self::Fiq => write!(f, "fiq"),
            Self::Irq => write!(f, "irq"),
            Self::Supervisor => write!(f, "svc"),
            Self::Abort => write!(f, "abt"),
            Self::Undefined => write!(f, "und"),
            Self::System => write!(f, "sys"),
            Self::Invalid => write!(f, "invalid mode"),
        }
    }
}

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum CpuState {
    #[base]
    Arm = 0,
    Thumb = 1,
}

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arm => write!(f, "ARM"),
            Self::Thumb => write!(f, "Thumb"),
        }
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

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::R0 => write!(f, "r0"),
            Self::R1 => write!(f, "r1"),
            Self::R2 => write!(f, "r2"),
            Self::R3 => write!(f, "r3"),
            Self::R4 => write!(f, "r4"),
            Self::R5 => write!(f, "r5"),
            Self::R6 => write!(f, "r6"),
            Self::R7 => write!(f, "r7"),
            Self::R8 => write!(f, "r8"),
            Self::R9 => write!(f, "r9"),
            Self::R10 => write!(f, "r10"),
            Self::R11 => write!(f, "r11"),
            Self::R12 => write!(f, "r12"),
            Self::R13 => write!(f, "sp"),
            Self::R14 => write!(f, "lr"),
            Self::R15 => write!(f, "pc"),
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

impl fmt::Display for LoRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::R0 => write!(f, "r0"),
            Self::R1 => write!(f, "r1"),
            Self::R2 => write!(f, "r2"),
            Self::R3 => write!(f, "r3"),
            Self::R4 => write!(f, "r4"),
            Self::R5 => write!(f, "r5"),
            Self::R6 => write!(f, "r6"),
            Self::R7 => write!(f, "r7"),
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

impl fmt::Display for HiRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::R8 => write!(f, "r8"),
            Self::R9 => write!(f, "r9"),
            Self::R10 => write!(f, "r10"),
            Self::R11 => write!(f, "r11"),
            Self::R12 => write!(f, "r12"),
            Self::R13 => write!(f, "sp"),
            Self::R14 => write!(f, "lr"),
            Self::R15 => write!(f, "pc"),
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
    NV,
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
            0b1111 => Self::NV,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EQ => write!(f, "EQ"),
            Self::NE => write!(f, "NE"),
            Self::CS => write!(f, "CS"),
            Self::CC => write!(f, "CC"),
            Self::MI => write!(f, "MI"),
            Self::PL => write!(f, "PL"),
            Self::VS => write!(f, "VS"),
            Self::VC => write!(f, "VC"),
            Self::HI => write!(f, "HI"),
            Self::LS => write!(f, "LS"),
            Self::GE => write!(f, "GE"),
            Self::LT => write!(f, "LT"),
            Self::GT => write!(f, "GT"),
            Self::LE => write!(f, "LE"),
            Self::AL => write!(f, ""),
            Self::NV => write!(f, "NV"),
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

impl fmt::Display for DataProcessingOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AND => write!(f, "AND"),
            Self::EOR => write!(f, "EOR"),
            Self::SUB => write!(f, "SUB"),
            Self::RSB => write!(f, "RSB"),
            Self::ADD => write!(f, "ADD"),
            Self::ADC => write!(f, "ADC"),
            Self::SBC => write!(f, "SBC"),
            Self::RSC => write!(f, "RSC"),
            Self::TST => write!(f, "TST"),
            Self::TEQ => write!(f, "TEQ"),
            Self::CMP => write!(f, "CMP"),
            Self::CMN => write!(f, "CMN"),
            Self::ORR => write!(f, "ORR"),
            Self::MOV => write!(f, "MOV"),
            Self::BIC => write!(f, "BIC"),
            Self::MVN => write!(f, "MVN"),
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

impl fmt::Display for MovCmpAddSubImmediateOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MOV => write!(f, "MOV"),
            Self::CMP => write!(f, "CMP"),
            Self::ADD => write!(f, "ADD"),
            Self::SUB => write!(f, "SUB"),
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

impl fmt::Display for AluOperationsOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AND => write!(f, "AND"),
            Self::EOR => write!(f, "EOR"),
            Self::LSL => write!(f, "LSL"),
            Self::LSR => write!(f, "LSR"),
            Self::ASR => write!(f, "ASR"),
            Self::ADC => write!(f, "ADC"),
            Self::SBC => write!(f, "SBC"),
            Self::ROR => write!(f, "ROR"),
            Self::TST => write!(f, "TST"),
            Self::NEG => write!(f, "NEG"),
            Self::CMP => write!(f, "CMP"),
            Self::CMN => write!(f, "CMN"),
            Self::ORR => write!(f, "ORR"),
            Self::MUL => write!(f, "MUL"),
            Self::BIC => write!(f, "BIC"),
            Self::MVN => write!(f, "MVN"),
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

impl fmt::Display for HiRegOpsBxOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ADD => write!(f, "ADD"),
            Self::CMP => write!(f, "CMP"),
            Self::MOV => write!(f, "MOV"),
            Self::BX => write!(f, "BX"),
        }
    }
}

#[allow(unused)]
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
