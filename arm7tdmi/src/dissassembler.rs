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
        use CpuMode::*;
        match mode {
            User => 0b10000,
            FIQ => 0b10001,
            IRQ => 0b10010,
            Supervisor => 0b10011,
            Abort => 0b10111,
            Undefined => 0b11011,
            System => 0b11111,
        }
    }
}

impl From<u32> for CpuMode {
    fn from(value: u32) -> Self {
        use CpuMode::*;
        match value {
            0b10000 => User,
            0b10001 => FIQ,
            0b10010 => IRQ,
            0b10011 => Supervisor,
            0b10111 => Abort,
            0b11011 => Undefined,
            0b11111 => System,
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
    Invalid,
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
            _ => Invalid,
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Condition::*;
        match self {
            EQ => write!(f, "EQ"),
            NE => write!(f, "NE"),
            CS => write!(f, "CS"),
            CC => write!(f, "CC"),
            MI => write!(f, "MI"),
            PL => write!(f, "PL"),
            VS => write!(f, "VS"),
            VC => write!(f, "VC"),
            HI => write!(f, "HI"),
            LS => write!(f, "LS"),
            GE => write!(f, "GE"),
            LT => write!(f, "LT"),
            GT => write!(f, "GT"),
            LE => write!(f, "LE"),
            AL => write!(f, ""),
            Invalid => panic!("Invalid condition"),
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

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Register::*;
        match self {
            R0 => write!(f, "r0"),
            R1 => write!(f, "r1"),
            R2 => write!(f, "r2"),
            R3 => write!(f, "r3"),
            R4 => write!(f, "r4"),
            R5 => write!(f, "r5"),
            R6 => write!(f, "r6"),
            R7 => write!(f, "r7"),
            R8 => write!(f, "r8"),
            R9 => write!(f, "r9"),
            R10 => write!(f, "r10"),
            R11 => write!(f, "r11"),
            R12 => write!(f, "r12"),
            R13 => write!(f, "sp"),
            R14 => write!(f, "lr"),
            R15 => write!(f, "pc"),
        }
    }
}
