use std::fmt;

use crate::arm::{Condition, DataProcessingInstructionKind};

pub mod branch;

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

impl fmt::Display for DataProcessingInstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DataProcessingInstructionKind::*;
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
