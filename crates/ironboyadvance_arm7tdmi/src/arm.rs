use crate::{
    Condition, DataProcessingOpcode, Register,
    barrel_shifter::{ShiftBy, ShiftType},
};
use getset::CopyGetters;
use ironboyadvance_common::bits::BitOps;

#[derive(Debug, Clone, Copy)]
pub enum ArmInstruction {
    DataProcessing(DataProcessing),
    PsrTransfer(PsrTransfer),
    Multiply(Multiply),
    MultiplyLong(MultiplyLong),
    SingleDataSwap(SingleDataSwap),
    BranchAndExchange(BranchAndExchange),
    HalfwordAndSignedDataTransfer(HalfwordAndSignedDataTransfer),
    SingleDataTransfer(SingleDataTransfer),
    Undefined(Undefined),
    BlockDataTransfer(BlockDataTransfer),
    BranchAndBranchWithLink(BranchAndBranchWithLink),
    SoftwareInterrupt(SoftwareInterrupt),
    CoprocessorDataOperation(Undefined),
    CoprocessorDataTransfer(Undefined),
    CoprocessorRegisterTransfer(Undefined),
}

impl ArmInstruction {
    pub fn cond(&self) -> Condition {
        match self {
            Self::DataProcessing(i) => i.cond(),
            Self::PsrTransfer(i) => i.cond(),
            Self::Multiply(i) => i.cond(),
            Self::MultiplyLong(i) => i.cond(),
            Self::SingleDataSwap(i) => i.cond(),
            Self::BranchAndExchange(i) => i.cond(),
            Self::HalfwordAndSignedDataTransfer(i) => i.cond(),
            Self::SingleDataTransfer(i) => i.cond(),
            Self::Undefined(i) => i.cond(),
            Self::BlockDataTransfer(i) => i.cond(),
            Self::BranchAndBranchWithLink(i) => i.cond(),
            Self::SoftwareInterrupt(i) => i.cond(),
            Self::CoprocessorDataOperation(i) => i.cond(),
            Self::CoprocessorDataTransfer(i) => i.cond(),
            Self::CoprocessorRegisterTransfer(i) => i.cond(),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct BlockDataTransfer {
    cond: Condition,
    rn: Register,
    pre_index: bool,
    add: bool,
    write_back: bool,
    load: bool,
    load_psr_force_user: bool,
    register_list: u16,
}

impl BlockDataTransfer {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            pre_index: value.bit(24),
            add: value.bit(23),
            write_back: value.bit(21),
            load: value.bit(20),
            load_psr_force_user: value.bit(22),
            register_list: value as u16,
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct BranchAndBranchWithLink {
    cond: Condition,
    link: bool,
    offset: u32,
}

impl BranchAndBranchWithLink {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            link: value.bit(24),
            offset: value.bits(0..=23),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct BranchAndExchange {
    cond: Condition,
    rn: Register,
}

impl BranchAndExchange {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(0..=3).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct DataProcessing {
    cond: Condition,
    rn: Register,
    rm: Register,
    rs: Register,
    rd: Register,
    is_immediate: bool,
    opcode: DataProcessingOpcode,
    sets_flags: bool,
    shift_by: ShiftBy,
    shift_amount: u32,
    shift_type: ShiftType,
    rotate: u32,
    immediate: u32,
}

impl DataProcessing {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            rm: value.bits(0..=3).into(),
            rs: value.bits(8..=11).into(),
            rd: value.bits(12..=15).into(),
            is_immediate: value.bit(25),
            opcode: value.bits(21..=24).into(),
            sets_flags: value.bit(20),
            shift_by: match value.bit(4) {
                true => ShiftBy::Register,
                false => ShiftBy::Immediate,
            },
            shift_amount: value.bits(7..=11),
            shift_type: value.bits(5..=6).into(),
            rotate: value.bits(8..=11),
            immediate: value.bits(0..=7),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct HalfwordAndSignedDataTransfer {
    cond: Condition,
    rn: Register,
    rd: Register,
    rm: Register,
    is_immediate: bool,
    immediate_hi: u32,
    immediate_lo: u32,
    pre_index: bool,
    add: bool,
    write_back: bool,
    load: bool,
    signed: bool,
    halfword: bool,
}

impl HalfwordAndSignedDataTransfer {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            is_immediate: value.bit(22),
            immediate_hi: value.bits(8..=11),
            immediate_lo: value.bits(0..=3),
            pre_index: value.bit(24),
            add: value.bit(23),
            write_back: value.bit(21),
            load: value.bit(20),
            signed: value.bit(6),
            halfword: value.bit(5),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MultiplyLong {
    cond: Condition,
    rd_hi: Register,
    rd_lo: Register,
    rm: Register,
    rs: Register,
    sets_flags: bool,
    accumulate: bool,
    unsigned: bool,
}

impl MultiplyLong {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rd_hi: value.bits(16..=19).into(),
            rd_lo: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            rs: value.bits(8..=11).into(),
            sets_flags: value.bit(20),
            accumulate: value.bit(21),
            unsigned: value.bit(22),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Multiply {
    cond: Condition,
    rn: Register,
    rd: Register,
    rm: Register,
    rs: Register,
    sets_flags: bool,
    accumulate: bool,
}

impl Multiply {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(12..=15).into(),
            rd: value.bits(16..=19).into(),
            rm: value.bits(0..=3).into(),
            rs: value.bits(8..=11).into(),
            sets_flags: value.bit(20),
            accumulate: value.bit(21),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PsrTransfer {
    cond: Condition,
    is_mrs: bool,
    psr_mask: u32,
    rd: Register,
    rm: Register,
    is_immediate: bool,
    rotate: u32,
    immediate: u32,
    is_spsr: bool,
}

impl PsrTransfer {
    #[inline]
    pub fn new(value: u32) -> Self {
        let field_bits = value.bits(16..=19);
        let mut psr_mask = 0u32;
        if field_bits & 0b1000 != 0 {
            psr_mask |= 0xFF000000;
        }
        if field_bits & 0b0100 != 0 {
            psr_mask |= 0xFF0000;
        }
        if field_bits & 0b0010 != 0 {
            psr_mask |= 0xFF00;
        }
        if field_bits & 0b0001 != 0 {
            psr_mask |= 0xFF;
        }

        Self {
            cond: value.bits(28..=31).into(),
            is_mrs: value.bits(16..=21) as u8 == 0xF,
            psr_mask,
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            is_immediate: value.bit(25),
            rotate: value.bits(8..=11),
            immediate: value.bits(0..=7),
            is_spsr: value.bit(22),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SingleDataSwap {
    cond: Condition,
    rn: Register,
    rd: Register,
    rm: Register,
    byte: bool,
}

impl SingleDataSwap {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            byte: value.bit(22),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SingleDataTransfer {
    cond: Condition,
    rn: Register,
    rd: Register,
    rm: Register,
    is_immediate: bool,
    shift_amount: u32,
    shift_type: ShiftType,
    immediate: u32,
    pre_index: bool,
    add: bool,
    byte: bool,
    write_back: bool,
    load: bool,
}

impl SingleDataTransfer {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            is_immediate: !value.bit(25),
            shift_amount: value.bits(7..=11),
            shift_type: value.bits(5..=6).into(),
            immediate: value.bits(0..=11),
            pre_index: value.bit(24),
            add: value.bit(23),
            byte: value.bit(22),
            write_back: value.bit(21),
            load: value.bit(20),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SoftwareInterrupt {
    cond: Condition,
    comment: u32,
}

impl SoftwareInterrupt {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            comment: value.bits(0..=23),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Undefined {
    cond: Condition,
}

impl Undefined {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
        }
    }
}
