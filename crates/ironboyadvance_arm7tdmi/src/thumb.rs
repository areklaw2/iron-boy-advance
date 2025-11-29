use ThumbInstructionKind::*;
use disassembler::*;
use execute::*;
use ironboyadvance_utils::bit::BitOps;
use std::fmt;

use crate::{
    Condition, CpuAction, HiRegister, LoRegister,
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
    value: u16,
    executed_pc: u32,
}

impl fmt::Display for ThumbInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThumbInstruction: kind: {:?}, bits: {} -> (0x{:04X}), executed_pc:{} -> (0x{:08X})",
            self.kind, self.value, self.value, self.executed_pc, self.executed_pc
        )
    }
}

impl Instruction for ThumbInstruction {
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
            AddOffsetToSp => disassemble_add_offset_to_sp(self),
            PushPopRegisters => disassemble_push_pop_registers(self),
            MultipleLoadStore => disassemble_multiple_load_store(self),
            ConditionalBranch => disassemble_conditional_branch(self),
            SoftwareInterrupt => disassemble_software_interrupt(self),
            UnconditionalBranch => disassemble_unconditional_branch(self),
            LongBranchWithLink => disassemble_long_branch_with_link(self),
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
            AddOffsetToSp => execute_add_offset_to_sp(cpu, self),
            PushPopRegisters => execute_push_pop_registers(cpu, self),
            MultipleLoadStore => execute_multiple_load_store(cpu, self),
            ConditionalBranch => execute_conditional_branch(cpu, self),
            SoftwareInterrupt => execute_software_interrupt(cpu, self),
            UnconditionalBranch => execute_unconditional_branch(cpu, self),
            LongBranchWithLink => execute_long_branch_with_link(cpu, self),
            Undefined => unimplemented!(),
        }
    }
}

impl ThumbInstruction {
    pub fn new(kind: ThumbInstructionKind, instruction: u16, executed_pc: u32) -> ThumbInstruction {
        ThumbInstruction {
            kind,
            value: instruction,
            executed_pc,
        }
    }

    pub fn opcode(&self) -> u16 {
        match self.kind {
            MoveShiftedRegister | MoveCompareAddSubtractImmediate => self.value.bits(11..=12),
            AddSubtract => self.value.bit(9) as u16,
            AluOperations => self.value.bits(6..=9),
            HiRegisterOperationsBranchExchange => self.value.bits(8..=9),
            _ => unimplemented!(),
        }
    }

    pub fn offset(&self) -> u16 {
        match self.kind {
            MoveShiftedRegister | LoadStoreImmediateOffset | LoadStoreHalfword => self.value.bits(6..=10),
            AddSubtract => self.value.bits(6..=8),
            MoveCompareAddSubtractImmediate
            | PcRelativeLoad
            | SpRelativeLoadStore
            | LoadAddress
            | ConditionalBranch
            | SoftwareInterrupt => self.value.bits(0..=7),
            AddOffsetToSp => self.value.bits(0..=6),
            UnconditionalBranch | LongBranchWithLink => self.value.bits(0..=10),
            _ => unimplemented!(),
        }
    }

    pub fn is_immediate(&self) -> bool {
        self.value.bit(10)
    }

    pub fn rn(&self) -> LoRegister {
        self.value.bits(6..=8).into()
    }

    pub fn rs(&self) -> LoRegister {
        self.value.bits(3..=5).into()
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
            | LoadStoreHalfword => self.value.bits(0..=2).into(),
            MoveCompareAddSubtractImmediate | PcRelativeLoad | SpRelativeLoadStore | LoadAddress => {
                self.value.bits(8..=10).into()
            }
            _ => unimplemented!(),
        }
    }

    pub fn rb(&self) -> LoRegister {
        match self.kind {
            LoadStoreRegisterOffset | LoadStoreSignExtendedByteHalfword | LoadStoreImmediateOffset | LoadStoreHalfword => {
                self.value.bits(3..=5).into()
            }
            MultipleLoadStore => self.value.bits(8..=10).into(),
            _ => unimplemented!(),
        }
    }

    pub fn ro(&self) -> LoRegister {
        self.value.bits(6..=8).into()
    }

    pub fn hs(&self) -> HiRegister {
        self.value.bits(3..=5).into()
    }

    pub fn hd(&self) -> HiRegister {
        self.value.bits(0..=2).into()
    }

    pub fn h1(&self) -> bool {
        self.value.bit(7)
    }

    pub fn h2(&self) -> bool {
        self.value.bit(6)
    }

    pub fn load(&self) -> bool {
        self.value.bit(11)
    }

    pub fn byte(&self) -> bool {
        match self.kind {
            LoadStoreRegisterOffset => self.value.bit(10),
            LoadStoreImmediateOffset => self.value.bit(12),
            _ => unimplemented!(),
        }
    }

    pub fn halfword(&self) -> bool {
        self.value.bit(11)
    }

    pub fn signed(&self) -> bool {
        match self.kind {
            LoadStoreSignExtendedByteHalfword => self.value.bit(10),
            AddOffsetToSp => self.value.bit(7),
            _ => unimplemented!(),
        }
    }

    pub fn sp(&self) -> bool {
        self.value.bit(11)
    }

    pub fn store_lr_load_pc(&self) -> bool {
        self.value.bit(8)
    }

    pub fn register_list(&self) -> Vec<usize> {
        (0..=7).filter(|&i| self.value.bit(i)).collect()
    }

    pub fn cond(&self) -> Condition {
        (self.value.bits(8..=11) as u32).into()
    }

    pub fn high(&self) -> bool {
        self.value.bit(11)
    }
}
