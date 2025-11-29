use ArmInstructionKind::*;
use core::fmt;
use disassembler::*;
use execute::*;
use ironboyadvance_utils::bit::BitOps;

use crate::{
    Condition, CpuAction, DataProcessingOpcode,
    barrel_shifter::{ShiftBy, ShiftType},
    cpu::Arm7tdmiCpu,
    memory::MemoryInterface,
};

use super::{Register, cpu::Instruction};

pub mod disassembler;
pub mod execute;
pub mod lut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArmInstructionKind {
    DataProcessing,
    PsrTransfer,
    Multiply,
    MultiplyLong,
    SingleDataSwap,
    BranchAndExchange,
    HalfwordAndSignedDataTransfer,
    SingleDataTransfer,
    Undefined,
    BlockDataTransfer,
    BranchAndBranchWithLink,
    SoftwareInterrupt,
    //CoprocessorDataTransfer,
    //CoprocessorDataOperation,
    //CoprocessorRegisterTransfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmInstruction {
    kind: ArmInstructionKind,
    value: u32,
    executed_pc: u32,
}

impl fmt::Display for ArmInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ArmInstruction: kind: {:?}, bits: {} -> (0x{:08X}), executed_pc:{} -> (0x{:08X})",
            self.kind, self.value, self.value, self.executed_pc, self.executed_pc
        )
    }
}

impl Instruction for ArmInstruction {
    #[inline]
    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        match self.kind {
            BranchAndExchange => disassemble_branch_exchange(self),
            BranchAndBranchWithLink => disassemble_branch_and_branch_link(self),
            DataProcessing => disassemble_data_processing(self),
            PsrTransfer => disassemble_psr_transfer(cpu, self),
            Multiply => disassemble_multiply(self),
            MultiplyLong => disassemble_multiply_long(self),
            SingleDataTransfer => disassemble_single_data_transfer(self),
            HalfwordAndSignedDataTransfer => disassemble_halfword_and_signed_data_transfer(self),
            BlockDataTransfer => disassemble_block_data_transfer(self),
            SingleDataSwap => disassemble_single_data_swap(self),
            SoftwareInterrupt => disassemble_software_interrupt(self),
            Undefined => disassemble_undefined(self),
        }
    }

    #[inline]
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match self.kind {
            BranchAndExchange => execute_branch_exchange(cpu, self),
            BranchAndBranchWithLink => execute_branch_and_branch_link(cpu, self),
            DataProcessing => execute_data_processing(cpu, self),
            PsrTransfer => execute_psr_transfer(cpu, self),
            Multiply => execute_multiply(cpu, self),
            MultiplyLong => execute_multiply_long(cpu, self),
            SingleDataTransfer => execute_single_data_transfer(cpu, self),
            HalfwordAndSignedDataTransfer => execute_halfword_and_signed_data_transfer(cpu, self),
            BlockDataTransfer => execute_block_data_transfer(cpu, self),
            SingleDataSwap => execute_single_data_swap(cpu, self),
            SoftwareInterrupt => execute_software_interrupt(cpu, self),
            Undefined => execute_undefined(cpu, self),
        }
    }
}

impl ArmInstruction {
    #[inline]
    pub fn new(kind: ArmInstructionKind, instruction: u32, executed_pc: u32) -> ArmInstruction {
        ArmInstruction {
            kind,
            value: instruction,
            executed_pc,
        }
    }

    #[inline]
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }

    #[inline]
    pub fn rn(&self) -> Register {
        match self.kind {
            BranchAndExchange => self.value.bits(0..=3).into(),
            DataProcessing | SingleDataTransfer | HalfwordAndSignedDataTransfer | BlockDataTransfer | SingleDataSwap => {
                self.value.bits(16..=19).into()
            }
            Multiply => self.value.bits(12..=15).into(),
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn rd(&self) -> Register {
        match self.kind {
            PsrTransfer | DataProcessing | SingleDataTransfer | HalfwordAndSignedDataTransfer | SingleDataSwap => {
                self.value.bits(12..=15).into()
            }
            Multiply => self.value.bits(16..=19).into(),
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn rd_hi(&self) -> Register {
        self.value.bits(16..=19).into()
    }

    #[inline]
    pub fn rd_lo(&self) -> Register {
        self.value.bits(12..=15).into()
    }

    #[inline]
    pub fn rm(&self) -> Register {
        self.value.bits(0..=3).into()
    }

    #[inline]
    pub fn rs(&self) -> Register {
        self.value.bits(8..=11).into()
    }

    #[inline]
    pub fn link(&self) -> bool {
        self.value.bit(24)
    }

    #[inline]
    pub fn offset(&self) -> u32 {
        self.value.bits(0..=23)
    }

    #[inline]
    pub fn is_immediate(&self) -> bool {
        match self.kind {
            PsrTransfer | DataProcessing => self.value.bit(25),
            SingleDataTransfer => !self.value.bit(25),
            HalfwordAndSignedDataTransfer => self.value.bit(22),
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn opcode(&self) -> DataProcessingOpcode {
        self.value.bits(21..=24).into()
    }

    #[inline]
    pub fn sets_flags(&self) -> bool {
        self.value.bit(20)
    }

    #[inline]
    pub fn shift_by(&self) -> ShiftBy {
        match self.value.bit(4) {
            true => ShiftBy::Register,
            false => ShiftBy::Immediate,
        }
    }

    #[inline]
    pub fn shift_amount(&self) -> u32 {
        self.value.bits(7..=11)
    }

    #[inline]
    pub fn shift_type(&self) -> ShiftType {
        self.value.bits(5..=6).into()
    }

    #[inline]
    pub fn rotate(&self) -> u32 {
        self.value.bits(8..=11)
    }

    #[inline]
    pub fn immediate(&self) -> u32 {
        match self.kind {
            PsrTransfer | DataProcessing => self.value.bits(0..=7),
            SingleDataTransfer => self.value.bits(0..=11),
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn immediate_hi(&self) -> u32 {
        self.value.bits(8..=11)
    }

    #[inline]
    pub fn immediate_lo(&self) -> u32 {
        self.value.bits(0..=3)
    }

    #[inline]
    pub fn is_spsr(&self) -> bool {
        self.value.bit(22)
    }

    #[inline]
    pub fn accumulate(&self) -> bool {
        self.value.bit(21)
    }

    #[inline]
    pub fn unsigned(&self) -> bool {
        self.value.bit(22)
    }

    #[inline]
    pub fn pre_index(&self) -> bool {
        self.value.bit(24)
    }

    #[inline]
    pub fn add(&self) -> bool {
        self.value.bit(23)
    }

    #[inline]
    pub fn byte(&self) -> bool {
        self.value.bit(22)
    }

    #[inline]
    pub fn write_back(&self) -> bool {
        self.value.bit(21)
    }

    #[inline]
    pub fn load(&self) -> bool {
        self.value.bit(20)
    }

    #[inline]
    pub fn signed(&self) -> bool {
        self.value.bit(6)
    }

    #[inline]
    pub fn halfword(&self) -> bool {
        self.value.bit(5)
    }

    #[inline]
    pub fn load_psr_force_user(&self) -> bool {
        self.value.bit(22)
    }

    #[inline]
    pub fn register_list(&self) -> Vec<usize> {
        (0..=15).filter(|&i| self.value.bit(i)).collect()
    }

    #[inline]
    pub fn comment(&self) -> u32 {
        self.value.bits(0..=23)
    }
}
