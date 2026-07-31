use std::fmt;

pub mod cpu;
mod instruction;
pub mod memory;
mod registers;

pub const CPU_CLOCK_SPEED: u32 = 4194304;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum GbMode {
    Monochrome,
    Color,
    ColorAsMonochrome,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum GbSpeed {
    Normal,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register8 {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    HLMem = 0b110,
    A = 0b111,
}

impl From<u8> for Register8 {
    fn from(value: u8) -> Self {
        match value {
            0b000 => Register8::B,
            0b001 => Register8::C,
            0b010 => Register8::D,
            0b011 => Register8::E,
            0b100 => Register8::H,
            0b101 => Register8::L,
            0b110 => Register8::HLMem,
            0b111 => Register8::A,
            _ => panic!("Invalid value was passed"),
        }
    }
}

impl fmt::Display for Register8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Register8::A => write!(f, "A"),
            Register8::B => write!(f, "B"),
            Register8::C => write!(f, "C"),
            Register8::D => write!(f, "D"),
            Register8::E => write!(f, "E"),
            Register8::H => write!(f, "H"),
            Register8::L => write!(f, "L"),
            Register8::HLMem => write!(f, "(HL)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register16 {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11,
}

impl From<u8> for Register16 {
    fn from(value: u8) -> Register16 {
        match value {
            0b00 => Register16::BC,
            0b01 => Register16::DE,
            0b10 => Register16::HL,
            0b11 => Register16::SP,
            _ => panic!("Invalid value was passed"),
        }
    }
}

impl fmt::Display for Register16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Register16::BC => write!(f, "BC"),
            Register16::DE => write!(f, "DE"),
            Register16::HL => write!(f, "HL"),
            Register16::SP => write!(f, "SP"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register16Stack {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    AF = 0b11,
}

impl From<u8> for Register16Stack {
    fn from(value: u8) -> Register16Stack {
        match value {
            0b00 => Register16Stack::BC,
            0b01 => Register16Stack::DE,
            0b10 => Register16Stack::HL,
            0b11 => Register16Stack::AF,
            _ => panic!("Invalid value was passed"),
        }
    }
}

impl fmt::Display for Register16Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Register16Stack::BC => write!(f, "BC"),
            Register16Stack::DE => write!(f, "DE"),
            Register16Stack::HL => write!(f, "HL"),
            Register16Stack::AF => write!(f, "AF"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R16Memory {
    BC = 0b00,
    DE = 0b01,
    HLI = 0b10,
    HLD = 0b11,
}

impl From<u8> for R16Memory {
    fn from(value: u8) -> R16Memory {
        match value {
            0b00 => R16Memory::BC,
            0b01 => R16Memory::DE,
            0b10 => R16Memory::HLI,
            0b11 => R16Memory::HLD,
            _ => panic!("Invalid value was passed"),
        }
    }
}

impl fmt::Display for R16Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            R16Memory::BC => write!(f, "BC"),
            R16Memory::DE => write!(f, "DE"),
            R16Memory::HLI => write!(f, "HL+"),
            R16Memory::HLD => write!(f, "HL-"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Condition {
    NZ = 0b00,
    Z = 0b01,
    NC = 0b10,
    C = 0b11,
}

impl From<u8> for Condition {
    fn from(value: u8) -> Condition {
        match value {
            0b000 => Condition::NZ,
            0b001 => Condition::Z,
            0b010 => Condition::NC,
            0b011 => Condition::C,
            _ => panic!("Invalid value was passed"),
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Condition::NZ => write!(f, "NZ"),
            Condition::Z => write!(f, "Z"),
            Condition::NC => write!(f, "NC"),
            Condition::C => write!(f, "C"),
        }
    }
}
