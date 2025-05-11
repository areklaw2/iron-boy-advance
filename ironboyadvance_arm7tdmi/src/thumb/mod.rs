use ThumbInstructionKind::*;
use bitvec::{field::BitField, order::Lsb0, vec::BitVec, view::BitView};
use disassembler::*;
use execute::*;
use std::fmt;

use crate::{
    CpuAction, Register,
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
    HighRegisterOperationsOrBranchExchange,
    PcRelativeLoad,
    LoadStoreRegisterOffset,
    LoadStoreSignExtendedByteHalfword,
    LoadStoreImmediateOffset,
    LoadStoreHalfword,
    SpRelativeLoadStore,
    LoadAddress,
    AddOffsetToSp,
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
            "ThumbInstruction: kind: {:?}, bits: {} -> (0x{:04X}), executed_pc:{} -> (0x{:08X})",
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

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        match self.kind {
            MoveShiftedRegister => disassemble_move_shifted_register(self),
            AddSubtract => disassemble_add_subtract(self),
            MoveCompareAddSubtractImmediate => todo!(),
            AluOperations => todo!(),
            HighRegisterOperationsOrBranchExchange => todo!(),
            PcRelativeLoad => todo!(),
            LoadStoreRegisterOffset => todo!(),
            LoadStoreSignExtendedByteHalfword => todo!(),
            LoadStoreImmediateOffset => todo!(),
            LoadStoreHalfword => todo!(),
            SpRelativeLoadStore => todo!(),
            LoadAddress => todo!(),
            AddOffsetToSp => todo!(),
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
            MoveShiftedRegister => execute_move_shifted_register(cpu, self),
            AddSubtract => execute_add_subtract(cpu, self),
            MoveCompareAddSubtractImmediate => todo!(),
            AluOperations => todo!(),
            HighRegisterOperationsOrBranchExchange => todo!(),
            PcRelativeLoad => todo!(),
            LoadStoreRegisterOffset => todo!(),
            LoadStoreSignExtendedByteHalfword => todo!(),
            LoadStoreImmediateOffset => todo!(),
            LoadStoreHalfword => todo!(),
            SpRelativeLoadStore => todo!(),
            LoadAddress => todo!(),
            AddOffsetToSp => todo!(),
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

    pub fn opcode(&self) -> u16 {
        match self.kind {
            MoveShiftedRegister => self.bits[11..=12].load::<u16>(),
            AddSubtract => self.bits[9] as u16,
            _ => unimplemented!(),
        }
    }

    pub fn offset3(&self) -> u16 {
        self.bits[6..=8].load()
    }

    pub fn offset5(&self) -> u16 {
        self.bits[6..=10].load::<u16>().into()
    }

    pub fn rn(&self) -> Register {
        self.bits[6..=8].load::<u16>().into()
    }

    pub fn rs(&self) -> Register {
        match self.kind {
            MoveShiftedRegister | AddSubtract => self.bits[3..=5].load::<u16>().into(),
            _ => unimplemented!(),
        }
    }

    pub fn rd(&self) -> Register {
        match self.kind {
            MoveShiftedRegister | AddSubtract => self.bits[0..=2].load::<u16>().into(),
            _ => unimplemented!(),
        }
    }

    pub fn is_immediate(&self) -> bool {
        self.bits[10]
    }
}
