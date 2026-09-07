use getset::CopyGetters;
use ironboyadvance_common::bits::BitOps;

use crate::{
    AluOperationsOpcode, Condition, Dissasemble, HiRegOpsBxOpcode, HiRegister, LoRegister, MovCmpAddSubImmediateOpcode,
    barrel_shifter::ShiftType, cpu::Arm7tdmiCpu, memory::MemoryInterface,
};

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

impl Dissasemble for ThumbInstruction {
    #[inline]
    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        match self {
            Self::MoveShiftedRegister(i) => i.disassemble(cpu),
            Self::AddSubtract(i) => i.disassemble(cpu),
            Self::MoveCompareAddSubtractImmediate(i) => i.disassemble(cpu),
            Self::AluOperations(i) => i.disassemble(cpu),
            Self::HiRegisterOperationsBranchExchange(i) => i.disassemble(cpu),
            Self::PcRelativeLoad(i) => i.disassemble(cpu),
            Self::LoadStoreRegisterOffset(i) => i.disassemble(cpu),
            Self::LoadStoreSignExtendedByteHalfword(i) => i.disassemble(cpu),
            Self::LoadStoreImmediateOffset(i) => i.disassemble(cpu),
            Self::LoadStoreHalfword(i) => i.disassemble(cpu),
            Self::SpRelativeLoadStore(i) => i.disassemble(cpu),
            Self::LoadAddress(i) => i.disassemble(cpu),
            Self::AddOffsetToSp(i) => i.disassemble(cpu),
            Self::PushPopRegisters(i) => i.disassemble(cpu),
            Self::MultipleLoadStore(i) => i.disassemble(cpu),
            Self::ConditionalBranch(i) => i.disassemble(cpu),
            Self::SoftwareInterrupt(i) => i.disassemble(cpu),
            Self::UnconditionalBranch(i) => i.disassemble(cpu),
            Self::LongBranchWithLink(i) => i.disassemble(cpu),
            Self::Undefined(i) => i.disassemble(cpu),
        }
    }
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

impl Dissasemble for MoveShiftedRegister {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let shift_type = ShiftType::from(self.opcode);
        let offset5 = self.offset;
        let rs = self.rs;
        let rd = self.rd;
        format!("{} {},{},#{}", shift_type, rd, rs, offset5)
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

impl Dissasemble for AddSubtract {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rs = self.rs;
        let rd = self.rd;
        let is_immediate = self.is_immediate;
        let operand = match is_immediate {
            true => format!("#{}", self.offset),
            false => format!("{}", self.rn),
        };
        let opcode = match self.opcode != 0 {
            true => "SUB",
            false => "ADD",
        };
        format!("{} {},{},{}", opcode, rd, rs, operand)
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

impl Dissasemble for MoveCompareAddSubtractImmediate {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd;
        let offset = self.offset;
        let opcode = MovCmpAddSubImmediateOpcode::from(self.opcode);
        format!("{} {},#{}", opcode, rd, offset)
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

impl Dissasemble for AluOperations {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd;
        let rs = self.rs;
        let opcode = AluOperationsOpcode::from(self.opcode);
        format!("{} {},{}", opcode, rd, rs)
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

impl Dissasemble for HiRegisterOperationsBranchExchange {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let destination = match self.h1 {
            true => self.hd.to_string(),
            false => self.rd.to_string(),
        };

        let source = match self.h2 {
            true => self.hs.to_string(),
            false => self.rs.to_string(),
        };

        let opcode = HiRegOpsBxOpcode::from(self.opcode);
        format!("{} {},{}", opcode, destination, source)
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

impl Dissasemble for PcRelativeLoad {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd;
        let offset = self.offset;
        format!("LDR {},[PC, #{}]", rd, offset)
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

impl Dissasemble for LoadStoreRegisterOffset {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let byte = if self.byte { "B" } else { "" };
        let ro = self.ro;
        let rb = self.rb;
        let rd = self.rd;

        match self.load {
            true => format!("LDR{} {}, [{},{}]", byte, rd, rb, ro),
            false => format!("STR{} {}, [{},{}]", byte, rd, rb, ro),
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

impl Dissasemble for LoadStoreSignExtendedByteHalfword {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let ro = self.ro;
        let rb = self.rb;
        let rd = self.rd;
        let signed = self.signed;
        let halfword = self.halfword;
        match (signed, halfword) {
            (false, false) => format!("STRH {}, [{},{}]", rd, rb, ro),
            (false, true) => format!("LDRH {}, [{},{}]", rd, rb, ro),
            (true, false) => format!("LDSB {}, [{},{}]", rd, rb, ro),
            (true, true) => format!("LDSH {}, [{},{}]", rd, rb, ro),
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

impl Dissasemble for LoadStoreImmediateOffset {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let byte = if self.byte { "B" } else { "" };
        let offset = self.offset;
        let rb = self.rb;
        let rd = self.rd;

        match self.load {
            true => format!("LDR{} {}, [{},#{}]", byte, rd, rb, offset),
            false => format!("STR{} {}, [{},#{}]", byte, rd, rb, offset),
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

impl Dissasemble for LoadStoreHalfword {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        let rb = self.rb;
        let rd = self.rd;
        match self.load {
            true => format!("LDRH {}, [{},#{}]", rd, rb, offset),
            false => format!("STRH {}, [{},#{}]", rd, rb, offset),
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

impl Dissasemble for SpRelativeLoadStore {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        let rd = self.rd;
        match self.load {
            true => format!("LDR {}, [sp,#{}]", rd, offset),
            false => format!("STRH {}, [sp,#{}]", rd, offset),
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

impl Dissasemble for LoadAddress {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        let rd = self.rd;
        let sp = if self.sp { "sp" } else { "pc" };
        format!("ADD {},{},{}", rd, sp, offset)
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

impl Dissasemble for AddOffsetToSp {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        let signed = if self.signed { "-" } else { "" };
        format!("ADD sp, {}{}", signed, offset)
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

impl Dissasemble for PushPopRegisters {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let load = self.load;
        let store_lr_load_pc = self.store_lr_load_pc;
        let register_list: Vec<usize> = (0..8).filter(|&i| (self.register_list_bits >> i) & 1 == 1).collect();
        let register_list_str = register_list
            .iter()
            .map(|register| LoRegister::from(*register as u16).to_string())
            .collect::<Vec<String>>()
            .join(",");

        match (load, store_lr_load_pc) {
            (false, false) => format!("PUSH {{{}}}", register_list_str),
            (false, true) => format!("PUSH {{{},lr}}", register_list_str),
            (true, false) => format!("POP {{{}}}", register_list_str),
            (true, true) => format!("POP {{{},pc}}", register_list_str),
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

impl Dissasemble for MultipleLoadStore {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rb = self.rb;
        let load = self.load;
        let register_list: Vec<usize> = (0..8).filter(|&i| (self.register_list_bits >> i) & 1 == 1).collect();
        let register_list_str = register_list
            .iter()
            .map(|register| LoRegister::from(*register as u16).to_string())
            .collect::<Vec<String>>()
            .join(",");

        match load {
            true => format!("LDMIA {}!,{{{}}}", rb, register_list_str),
            false => format!("STMIA {}!,{{{}}}", rb, register_list_str),
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

impl Dissasemble for ConditionalBranch {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let offset = self.offset;
        format!("B{} #{}", cond, offset)
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

impl Dissasemble for SoftwareInterrupt {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        format!("SWI #{}", offset)
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

impl Dissasemble for UnconditionalBranch {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        format!("B #{}", offset)
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

impl Dissasemble for LongBranchWithLink {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        let high = if self.high { "hi" } else { "lo" };
        format!("BL #{}({})", offset, high)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Undefined {}

impl Undefined {
    pub fn new(_value: u16) -> Self {
        Self {}
    }
}

impl Dissasemble for Undefined {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        "Undefined".into()
    }
}
