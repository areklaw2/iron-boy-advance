mod arm;
pub mod cpu;
mod disassembler;
mod psr;
mod tests;
mod thumb;

pub const CPU_CLOCK_SPEED: u32 = 16777216;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CpuAction {
    Advance(u8),
    PipelineFlush,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CpuMode {
    User = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
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
            _ => unreachable!(),
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
