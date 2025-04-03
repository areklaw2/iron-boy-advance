#[derive(Debug, Copy, Clone)]
pub enum ShiftType {
    LSL,
    LSR,
    ASR,
    ROR,
}

impl From<u32> for ShiftType {
    fn from(value: u32) -> Self {
        use ShiftType::*;
        match value {
            0b00 => LSL,
            0b01 => LSR,
            0b10 => ASR,
            0b11 => ROR,
            _ => unreachable!(),
        }
    }
}

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
            *carry = (value >> amount - 1) & 0b1 != 0;
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
        0..=31 => {
            *carry = (value >> amount - 1) & 0b1 != 0;
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
        let amount = amount % 32;
        let value = if amount != 0 { value.rotate_right(amount) } else { value };
        *carry = value & (1 << 31) != 0;
        value
    }
}
