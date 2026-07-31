use std::fmt;

#[derive(Debug, Copy, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum ShiftType {
    LSL,
    LSR,
    ASR,
    ROR,
}

impl From<u32> for ShiftType {
    fn from(value: u32) -> Self {
        match value {
            0b00 => Self::LSL,
            0b01 => Self::LSR,
            0b10 => Self::ASR,
            0b11 => Self::ROR,
            _ => unreachable!(),
        }
    }
}

impl From<u16> for ShiftType {
    fn from(value: u16) -> Self {
        match value {
            0b00 => Self::LSL,
            0b01 => Self::LSR,
            0b10 => Self::ASR,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for ShiftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LSL => write!(f, "LSL"),
            Self::LSR => write!(f, "LSR"),
            Self::ASR => write!(f, "ASR"),
            Self::ROR => write!(f, "ROR"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShiftBy {
    Immediate,
    Register,
}

impl From<ShiftBy> for bool {
    fn from(value: ShiftBy) -> Self {
        match value {
            ShiftBy::Immediate => true,
            ShiftBy::Register => false,
        }
    }
}

pub fn lsl(value: u32, amount: u32, carry: &mut bool) -> u32 {
    match amount {
        0 => value,
        1..=31 => {
            *carry = (value << (amount - 1)) >> 31 != 0;
            value << amount
        }
        32 => {
            *carry = value & 0b1 != 0;
            0
        }
        _ => {
            *carry = false;
            0
        }
    }
}

pub fn lsr(value: u32, amount: u32, carry: &mut bool, is_immediate: bool) -> u32 {
    let amount = if is_immediate && amount == 0 { 32 } else { amount };
    match amount {
        0 => value,
        1..=31 => {
            *carry = (value >> (amount - 1)) & 0b1 != 0;
            value >> amount
        }
        32 => {
            *carry = value & (1 << 31) != 0;
            0
        }
        _ => {
            *carry = false;
            0
        }
    }
}

pub fn asr(value: u32, amount: u32, carry: &mut bool, is_immediate: bool) -> u32 {
    let amount = if is_immediate && amount == 0 { 32 } else { amount };
    match amount {
        0 => value,
        1..=31 => {
            *carry = (value >> (amount - 1)) & 0b1 != 0;
            ((value as i32) >> amount) as u32
        }
        _ => {
            let msb = value & (1 << 31) != 0;
            *carry = msb;
            match msb {
                true => u32::MAX,
                false => 0,
            }
        }
    }
}

pub fn ror(value: u32, amount: u32, carry: &mut bool, is_immediate: bool) -> u32 {
    if is_immediate && amount == 0 {
        //ror #0 -> rrx #1
        let curr_carry = *carry as u32;
        *carry = value & 0b1 != 0;
        (value >> 1) | (curr_carry) << 31
    } else {
        if amount == 0 {
            return value;
        }
        let amount = amount % 32;
        let value = if amount != 0 { value.rotate_right(amount) } else { value };
        *carry = value >> 31 != 0;
        value
    }
}
