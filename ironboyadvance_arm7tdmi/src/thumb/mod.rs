use ThumbInstructionKind::*;
use bitvec::{field::BitField, order::Lsb0, vec::BitVec, view::BitView};
use disassembler::*;
use execute::*;
use std::fmt;

use crate::{
    CpuAction, HiRegister, LoRegister,
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
    HiRegisterOperationsBranchExchange,
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

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        match self.kind {
            MoveShiftedRegister => disassemble_move_shifted_register(self),
            AddSubtract => disassemble_add_subtract(self),
            MoveCompareAddSubtractImmediate => disassemble_move_compare_add_subtract_immediate(self),
            AluOperations => disassemble_alu_operations(self),
            HiRegisterOperationsBranchExchange => disassemble_hi_register_operations_branch_exchange(self),
            PcRelativeLoad => disassemble_pc_relative_load(self),
            LoadStoreRegisterOffset => disassemble_load_store_register_offset(self),
            LoadStoreSignExtendedByteHalfword => disassemble_load_store_sign_extended_byte_halfword(self),
            LoadStoreImmediateOffset => disassemble_load_store_immediate_offset(self),
            LoadStoreHalfword => disassemble_load_store_halfword(self),
            SpRelativeLoadStore => disassemble_sp_relative_load_store(self),
            LoadAddress => disassemble_load_address(self),
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
            MoveCompareAddSubtractImmediate => execute_move_compare_add_subtract_immediate(cpu, self),
            AluOperations => execute_alu_operations(cpu, self),
            HiRegisterOperationsBranchExchange => execute_hi_register_operations_branch_exchange(cpu, self),
            PcRelativeLoad => execute_pc_relative_load(cpu, self),
            LoadStoreRegisterOffset => execute_load_store_register_offset(cpu, self),
            LoadStoreSignExtendedByteHalfword => execute_load_store_sign_extended_byte_halfword(cpu, self),
            LoadStoreImmediateOffset => execute_load_store_immediate_offset(cpu, self),
            LoadStoreHalfword => execute_load_store_halfword(cpu, self),
            SpRelativeLoadStore => execute_sp_relative_load_store(cpu, self),
            LoadAddress => execute_load_address(cpu, self),
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
            MoveShiftedRegister | MoveCompareAddSubtractImmediate => self.bits[11..=12].load(),
            AddSubtract => self.bits[9] as u16,
            AluOperations => self.bits[6..=9].load(),
            HiRegisterOperationsBranchExchange => self.bits[8..=9].load(),
            _ => unimplemented!(),
        }
    }

    pub fn offset(&self) -> u16 {
        match self.kind {
            MoveShiftedRegister | LoadStoreImmediateOffset | LoadStoreHalfword => self.bits[6..=10].load(),
            AddSubtract => self.bits[6..=8].load(),
            MoveCompareAddSubtractImmediate | PcRelativeLoad | SpRelativeLoadStore | LoadAddress => self.bits[0..=7].load(),
            _ => unimplemented!(),
        }
    }

    pub fn is_immediate(&self) -> bool {
        self.bits[10]
    }

    pub fn rn(&self) -> LoRegister {
        self.bits[6..=8].load::<u16>().into()
    }

    pub fn rs(&self) -> LoRegister {
        match self.kind {
            MoveShiftedRegister | AddSubtract | AluOperations | HiRegisterOperationsBranchExchange => {
                self.bits[3..=5].load::<u16>().into()
            }
            _ => unimplemented!(),
        }
    }

    pub fn rd(&self) -> LoRegister {
        match self.kind {
            MoveShiftedRegister
            | AddSubtract
            | AluOperations
            | HiRegisterOperationsBranchExchange
            | LoadStoreRegisterOffset
            | LoadStoreSignExtendedByteHalfword
            | LoadStoreImmediateOffset
            | LoadStoreHalfword => self.bits[0..=2].load::<u16>().into(),
            MoveCompareAddSubtractImmediate | PcRelativeLoad | SpRelativeLoadStore | LoadAddress => {
                self.bits[8..=10].load::<u16>().into()
            }
            _ => unimplemented!(),
        }
    }

    pub fn rb(&self) -> LoRegister {
        match self.kind {
            LoadStoreRegisterOffset | LoadStoreSignExtendedByteHalfword | LoadStoreImmediateOffset | LoadStoreHalfword => {
                self.bits[3..=5].load::<u16>().into()
            }
            _ => unimplemented!(),
        }
    }

    pub fn ro(&self) -> LoRegister {
        match self.kind {
            LoadStoreRegisterOffset | LoadStoreSignExtendedByteHalfword => self.bits[6..=8].load::<u16>().into(),
            _ => unimplemented!(),
        }
    }

    pub fn hs(&self) -> HiRegister {
        match self.kind {
            HiRegisterOperationsBranchExchange => self.bits[3..=5].load::<u16>().into(),
            _ => unimplemented!(),
        }
    }

    pub fn hd(&self) -> HiRegister {
        match self.kind {
            HiRegisterOperationsBranchExchange => self.bits[0..=2].load::<u16>().into(),
            _ => unimplemented!(),
        }
    }

    pub fn h1(&self) -> bool {
        self.bits[7]
    }

    pub fn h2(&self) -> bool {
        self.bits[6]
    }

    pub fn load(&self) -> bool {
        self.bits[11]
    }

    pub fn byte(&self) -> bool {
        match self.kind {
            LoadStoreRegisterOffset => self.bits[10],
            LoadStoreImmediateOffset => self.bits[12],
            _ => unimplemented!(),
        }
    }

    pub fn halfword(&self) -> bool {
        self.bits[11]
    }

    pub fn signed(&self) -> bool {
        self.bits[10]
    }

    pub fn sp(&self) -> bool {
        self.bits[11]
    }
}
