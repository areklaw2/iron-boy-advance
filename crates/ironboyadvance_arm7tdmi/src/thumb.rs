use crate::{Condition, HiRegister, LoRegister};
use getset::CopyGetters;
use ironboyadvance_common::bits::BitOps;

#[derive(Debug, Clone, Copy)]
pub enum ThumbInstruction {
    MoveShiftedRegister(MoveShiftedRegister),
    AddSubtract(AddSubtract),
    MoveCompareAddSubtractImmediate(MoveCompareAddSubtractImmediate),
    AluOperations(AluOperations),
    HiRegisterOperationsBranchExchange(HiRegisterOperationsBranchExchange),
    PcRelativeLoad(PcRelativeLoad),
    LoadStoreRegisterOffset(LoadStoreRegisterOffset),
    LoadStoreSignExtendedByteHalfword(LoadStoreSignExtendedByteHalfword),
    LoadStoreImmediateOffset(LoadStoreImmediateOffset),
    LoadStoreHalfword(LoadStoreHalfword),
    SpRelativeLoadStore(SpRelativeLoadStore),
    LoadAddress(LoadAddress),
    AddOffsetToSp(AddOffsetToSp),
    PushPopRegisters(PushPopRegisters),
    MultipleLoadStore(MultipleLoadStore),
    ConditionalBranch(ConditionalBranch),
    SoftwareInterrupt(SoftwareInterrupt),
    UnconditionalBranch(UnconditionalBranch),
    LongBranchWithLink(LongBranchWithLink),
    Undefined(Undefined),
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MoveShiftedRegister {
    rd: LoRegister,
    rs: LoRegister,
    offset: u16,
    opcode: u16,
}

impl MoveShiftedRegister {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rs: value.bits(3..=5).into(),
            offset: value.bits(6..=10),
            opcode: value.bits(11..=12),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct AddSubtract {
    rd: LoRegister,
    rs: LoRegister,
    rn: LoRegister,
    offset: u16,
    is_immediate: bool,
    opcode: u16,
}

impl AddSubtract {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rs: value.bits(3..=5).into(),
            rn: value.bits(6..=8).into(),
            offset: value.bits(6..=8),
            is_immediate: value.bit(10),
            opcode: value.bit(9) as u16,
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MoveCompareAddSubtractImmediate {
    rd: LoRegister,
    offset: u16,
    opcode: u16,
}

impl MoveCompareAddSubtractImmediate {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(8..=10).into(),
            offset: value.bits(0..=7),
            opcode: value.bits(11..=12),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct AluOperations {
    rd: LoRegister,
    rs: LoRegister,
    opcode: u16,
}

impl AluOperations {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rs: value.bits(3..=5).into(),
            opcode: value.bits(6..=9),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct HiRegisterOperationsBranchExchange {
    rd: LoRegister,
    hd: HiRegister,
    rs: LoRegister,
    hs: HiRegister,
    h1: bool,
    h2: bool,
    opcode: u16,
}

impl HiRegisterOperationsBranchExchange {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            hd: value.bits(0..=2).into(),
            rs: value.bits(3..=5).into(),
            hs: value.bits(3..=5).into(),
            h1: value.bit(7),
            h2: value.bit(6),
            opcode: value.bits(8..=9),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PcRelativeLoad {
    rd: LoRegister,
    offset: u16,
}

impl PcRelativeLoad {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(8..=10).into(),
            offset: value.bits(0..=7),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LoadStoreRegisterOffset {
    rd: LoRegister,
    rb: LoRegister,
    ro: LoRegister,
    byte: bool,
    load: bool,
}

impl LoadStoreRegisterOffset {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rb: value.bits(3..=5).into(),
            ro: value.bits(6..=8).into(),
            byte: value.bit(10),
            load: value.bit(11),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LoadStoreSignExtendedByteHalfword {
    rd: LoRegister,
    rb: LoRegister,
    ro: LoRegister,
    signed: bool,
    halfword: bool,
}

impl LoadStoreSignExtendedByteHalfword {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rb: value.bits(3..=5).into(),
            ro: value.bits(6..=8).into(),
            signed: value.bit(10),
            halfword: value.bit(11),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LoadStoreImmediateOffset {
    rd: LoRegister,
    rb: LoRegister,
    offset: u16,
    load: bool,
    byte: bool,
}

impl LoadStoreImmediateOffset {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rb: value.bits(3..=5).into(),
            offset: value.bits(6..=10),
            load: value.bit(11),
            byte: value.bit(12),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LoadStoreHalfword {
    rd: LoRegister,
    rb: LoRegister,
    offset: u16,
    load: bool,
}

impl LoadStoreHalfword {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rb: value.bits(3..=5).into(),
            offset: value.bits(6..=10),
            load: value.bit(11),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SpRelativeLoadStore {
    rd: LoRegister,
    offset: u16,
    load: bool,
}

impl SpRelativeLoadStore {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(8..=10).into(),
            offset: value.bits(0..=7),
            load: value.bit(11),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LoadAddress {
    rd: LoRegister,
    offset: u16,
    sp: bool,
}

impl LoadAddress {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(8..=10).into(),
            offset: value.bits(0..=7),
            sp: value.bit(11),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct AddOffsetToSp {
    offset: u16,
    signed: bool,
}

impl AddOffsetToSp {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            offset: value.bits(0..=6),
            signed: value.bit(7),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PushPopRegisters {
    register_list_bits: u8,
    store_lr_load_pc: bool,
    load: bool,
}

impl PushPopRegisters {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            register_list_bits: value as u8,
            store_lr_load_pc: value.bit(8),
            load: value.bit(11),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MultipleLoadStore {
    rb: LoRegister,
    register_list_bits: u8,
    load: bool,
}

impl MultipleLoadStore {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rb: value.bits(8..=10).into(),
            register_list_bits: value as u8,
            load: value.bit(11),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct ConditionalBranch {
    cond: Condition,
    offset: u16,
}

impl ConditionalBranch {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            cond: (value.bits(8..=11) as u32).into(),
            offset: value.bits(0..=7),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SoftwareInterrupt {
    offset: u16,
}

impl SoftwareInterrupt {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            offset: value.bits(0..=7),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct UnconditionalBranch {
    offset: u16,
}

impl UnconditionalBranch {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            offset: value.bits(0..=10),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LongBranchWithLink {
    offset: u16,
    high: bool,
}

impl LongBranchWithLink {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            offset: value.bits(0..=10),
            high: value.bit(11),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Undefined {}

impl Undefined {
    pub fn new(_value: u16) -> Self {
        Self {}
    }
}
