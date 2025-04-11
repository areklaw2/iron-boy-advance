use std::fmt::{self, write};

use crate::{CpuMode, CpuState, Register, alu::AluInstruction, arm::Condition, barrel_shifter::ShiftType};

impl fmt::Display for CpuMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CpuMode::*;
        match self {
            User => write!(f, "usr"),
            Fiq => write!(f, "fiq"),
            Irq => write!(f, "irq"),
            Supervisor => write!(f, "svc"),
            Abort => write!(f, "abt"),
            Undefined => write!(f, "und"),
            System => write!(f, "sys"),
            Invalid => write!(f, "invalid mode"),
        }
    }
}

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CpuState::*;
        match self {
            Arm => write!(f, "ARM"),
            Thumb => write!(f, "Thumb"),
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
        }
    }
}

impl fmt::Display for AluInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AluInstruction::*;
        match self {
            AND => write!(f, "AND"),
            EOR => write!(f, "EOR"),
            SUB => write!(f, "SUB"),
            RSB => write!(f, "RSB"),
            ADD => write!(f, "ADD"),
            ADC => write!(f, "ADC"),
            SBC => write!(f, "SBC"),
            RSC => write!(f, "RSC"),
            TST => write!(f, "TST"),
            TEQ => write!(f, "TEQ"),
            CMP => write!(f, "CMP"),
            CMN => write!(f, "CMN"),
            ORR => write!(f, "ORR"),
            MOV => write!(f, "MOV"),
            BIC => write!(f, "BIC"),
            MVN => write!(f, "MVN"),
        }
    }
}

impl fmt::Display for ShiftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ShiftType::*;
        match self {
            LSL => write!(f, "LSL"),
            LSR => write!(f, "LSR"),
            ASR => write!(f, "ASR"),
            ROR => write!(f, "ROR"),
        }
    }
}
