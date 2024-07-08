use std::{
    fmt::{self},
    panic,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CpuMode {
    User = 0b10000,
    FIQ = 0b10001,
    IRQ = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

impl From<CpuMode> for u32 {
    fn from(mode: CpuMode) -> u32 {
        match mode {
            CpuMode::User => 0b10000,
            CpuMode::FIQ => 0b10001,
            CpuMode::IRQ => 0b10010,
            CpuMode::Supervisor => 0b10011,
            CpuMode::Abort => 0b10111,
            CpuMode::Undefined => 0b11011,
            CpuMode::System => 0b11111,
        }
    }
}

impl From<u32> for CpuMode {
    fn from(value: u32) -> Self {
        match value {
            0b10000 => CpuMode::User,
            0b10001 => CpuMode::FIQ,
            0b10010 => CpuMode::IRQ,
            0b10011 => CpuMode::Supervisor,
            0b10111 => CpuMode::Abort,
            0b11011 => CpuMode::Undefined,
            0b11111 => CpuMode::System,
            _ => panic!("Invalid Cpu State"),
        }
    }
}

impl fmt::Display for CpuMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CpuMode::*;
        match self {
            User => write!(f, "usr"),
            FIQ => write!(f, "fiq"),
            IRQ => write!(f, "irq"),
            Supervisor => write!(f, "svc"),
            Abort => write!(f, "abt"),
            Undefined => write!(f, "und"),
            System => write!(f, "sys"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CpuState {
    ARM = 0,
    Thumb = 1,
}

impl From<CpuState> for bool {
    fn from(state: CpuState) -> bool {
        match state {
            CpuState::ARM => false,
            CpuState::Thumb => true,
        }
    }
}

impl From<bool> for CpuState {
    fn from(value: bool) -> Self {
        match value {
            false => CpuState::ARM,
            true => CpuState::Thumb,
        }
    }
}

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuState::ARM => write!(f, "ARM"),
            CpuState::Thumb => write!(f, "Thumb"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Condition {
    EQ,
    NE,
    HS,
    LO,
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
    Invalid,
}

impl From<u32> for Condition {
    fn from(value: u32) -> Self {
        match value {
            0b0000 => Condition::EQ,
            0b0001 => Condition::NE,
            0b0010 => Condition::HS,
            0b0011 => Condition::LO,
            0b0100 => Condition::MI,
            0b0101 => Condition::PL,
            0b0110 => Condition::VS,
            0b0111 => Condition::VC,
            0b1000 => Condition::HI,
            0b1001 => Condition::LS,
            0b1010 => Condition::GE,
            0b1011 => Condition::LT,
            0b1100 => Condition::GT,
            0b1101 => Condition::LE,
            0b1110 => Condition::AL,
            _ => Condition::Invalid,
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::EQ => write!(f, "eg"),
            Condition::NE => write!(f, "ne"),
            Condition::HS => write!(f, "hs"),
            Condition::LO => write!(f, "lo"),
            Condition::MI => write!(f, "mi"),
            Condition::PL => write!(f, "pl"),
            Condition::VS => write!(f, "vs"),
            Condition::VC => write!(f, "vc"),
            Condition::HI => write!(f, "hi"),
            Condition::LS => write!(f, "ls"),
            Condition::GE => write!(f, "ge"),
            Condition::LT => write!(f, "lt"),
            Condition::GT => write!(f, "gt"),
            Condition::LE => write!(f, "le"),
            Condition::AL => write!(f, ""),
            Condition::Invalid => panic!("Invalid condition"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Register {
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
            0b0000 => Register::R0,
            0b0001 => Register::R1,
            0b0010 => Register::R2,
            0b0011 => Register::R3,
            0b0100 => Register::R4,
            0b0101 => Register::R5,
            0b0110 => Register::R6,
            0b0111 => Register::R7,
            0b1000 => Register::R8,
            0b1001 => Register::R9,
            0b1010 => Register::R10,
            0b1011 => Register::R11,
            0b1100 => Register::R12,
            0b1101 => Register::R13,
            0b1110 => Register::R14,
            _ => Register::R15,
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::R0 => write!(f, "r0"),
            Register::R1 => write!(f, "r1"),
            Register::R2 => write!(f, "r2"),
            Register::R3 => write!(f, "r3"),
            Register::R4 => write!(f, "r4"),
            Register::R5 => write!(f, "r5"),
            Register::R6 => write!(f, "r6"),
            Register::R7 => write!(f, "r7"),
            Register::R8 => write!(f, "r8"),
            Register::R9 => write!(f, "r9"),
            Register::R10 => write!(f, "r10"),
            Register::R11 => write!(f, "r11"),
            Register::R12 => write!(f, "r12"),
            Register::R13 => write!(f, "sp"),
            Register::R14 => write!(f, "lr"),
            Register::R15 => write!(f, "pc"),
        }
    }
}
