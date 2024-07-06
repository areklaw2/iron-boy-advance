use std::fmt;

#[derive(Debug)]
pub enum DissassemblerError {
    CpuModeError,
    ConditionError,
}

impl DissassemblerError {}

impl fmt::Display for DissassemblerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DissassemblerError::CpuModeError => write!(f, "Invalid value passed to for CpuMode"),
            DissassemblerError::ConditionError => {
                write!(f, "Invalid value passed to for Condition")
            }
        }
    }
}

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
        use CpuState::*;
        match self {
            ARM => write!(f, "ARM"),
            Thumb => write!(f, "Thumb"),
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Condition {
    EQ = 0b0000,
    NE = 0b0001,
    HS = 0b0010,
    LO = 0b0011,
    MI = 0b0100,
    PL = 0b0101,
    VS = 0b0110,
    VC = 0b0111,
    HI = 0b1000,
    LS = 0b1001,
    GE = 0b1010,
    LT = 0b1011,
    GT = 0b1100,
    LE = 0b1101,
    AL = 0b1110,
    Invalid = 0b1111,
}
