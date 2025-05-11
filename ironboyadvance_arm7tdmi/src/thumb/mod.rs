use ThumbInstructionKind::*;
use bitvec::{field::BitField, order::Lsb0, vec::BitVec, view::BitView};
use std::fmt;

use crate::{
    CpuAction,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};

pub mod disassembler;
pub mod execute;
pub mod lut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ThumbInstructionKind {
    MoveShiftedRegister,
    AddSubtract,
    MoveCompareAddSubtractImmediate,
    AluOperations,
    HiRegisterOperationsOrBranchExchange,
    PcRelativeLoad,
    LoadStoreRegisterOffset,
    LoadStoreSignExtendedByteHalfword,
    LoadStoreImmediateOffset,
    LoadStoreHalfword,
    SpRelativeLoadStore,
    LoadAddress,
    AddOffsetSp,
    PushPopRegisters,
    MultipleLoadStore,
    ConditionalBranch,
    SoftwareInterrupt,
    UnconditionalBranch,
    LongBranchWithLink,
    Undefined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbInstruction {
    kind: ThumbInstructionKind,
    bits: BitVec<u16>,
    executed_pc: u32,
}

impl fmt::Display for ThumbInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThumbInstruction: kind: {:?}, bits: {} -> (0x{:08X}), executed_pc:{} -> (0x{:08X})",
            self.kind,
            self.bits.load::<u16>(),
            self.bits.load::<u16>(),
            self.executed_pc,
            self.executed_pc
        )
    }
}

impl Instruction for ThumbInstruction {
    type Size = u16;

    fn disassamble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        match self.kind {
            MoveShiftedRegister => todo!(),
            AddSubtract => todo!(),
            MoveCompareAddSubtractImmediate => todo!(),
            AluOperations => todo!(),
            HiRegisterOperationsOrBranchExchange => todo!(),
            PcRelativeLoad => todo!(),
            LoadStoreRegisterOffset => todo!(),
            LoadStoreSignExtendedByteHalfword => todo!(),
            LoadStoreImmediateOffset => todo!(),
            LoadStoreHalfword => todo!(),
            SpRelativeLoadStore => todo!(),
            LoadAddress => todo!(),
            AddOffsetSp => todo!(),
            PushPopRegisters => todo!(),
            MultipleLoadStore => todo!(),
            ConditionalBranch => todo!(),
            SoftwareInterrupt => todo!(),
            UnconditionalBranch => todo!(),
            LongBranchWithLink => todo!(),
            Undefined => unimplemented!(),
        }
    }

    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match self.kind {
            MoveShiftedRegister => todo!(),
            AddSubtract => todo!(),
            MoveCompareAddSubtractImmediate => todo!(),
            AluOperations => todo!(),
            HiRegisterOperationsOrBranchExchange => todo!(),
            PcRelativeLoad => todo!(),
            LoadStoreRegisterOffset => todo!(),
            LoadStoreSignExtendedByteHalfword => todo!(),
            LoadStoreImmediateOffset => todo!(),
            LoadStoreHalfword => todo!(),
            SpRelativeLoadStore => todo!(),
            LoadAddress => todo!(),
            AddOffsetSp => todo!(),
            PushPopRegisters => todo!(),
            MultipleLoadStore => todo!(),
            ConditionalBranch => todo!(),
            SoftwareInterrupt => todo!(),
            UnconditionalBranch => todo!(),
            LongBranchWithLink => todo!(),
            Undefined => unimplemented!(),
        }
    }

    fn value(&self) -> u16 {
        self.bits.load::<u16>()
    }
}

impl ThumbInstruction {
    pub fn new(kind: ThumbInstructionKind, instruction: u16, executed_pc: u32) -> ThumbInstruction {
        ThumbInstruction {
            kind,
            bits: instruction.view_bits::<Lsb0>().to_bitvec(),
            executed_pc,
        }
    }
}
