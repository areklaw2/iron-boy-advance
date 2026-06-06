use std::fmt;

use crate::{
    AluOperationsOpcode, Condition, CpuMode, CpuState, DataProcessingOpcode, HiRegOpsBxOpcode, HiRegister, LoRegister,
    MovCmpAddSubImmediateOpcode, Register, barrel_shifter::ShiftType,
};

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

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arm => write!(f, "ARM"),
            Self::Thumb => write!(f, "Thumb"),
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
