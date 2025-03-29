use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AluInstruction {
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

impl From<u32> for AluInstruction {
    fn from(value: u32) -> Self {
        use AluInstruction::*;
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
    Amount,
    Register,
}
