use core::fmt;

use crate::{
    Condition, CpuAction,
    arm::{
        block_data_transfer::BlockDataTransfer, branch_and_branch_link::BranchAndBranchWithLink,
        branch_and_exchange::BranchAndExchange, data_processing::DataProcessing,
        halfword_and_signed_data_transfer::HalfwordAndSignedDataTransfer, multiply::Multiply, multiply_long::MultiplyLong,
        psr_transfer::PsrTransfer, single_data_swap::SingleDataSwap, single_data_transfer::SingleDataTransfer,
        software_interrupt::SoftwareInterrupt, undefined::Undefined,
    },
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};

pub mod block_data_transfer;
pub mod branch_and_branch_link;
pub mod branch_and_exchange;
pub mod data_processing;
pub mod halfword_and_signed_data_transfer;
pub mod multiply;
pub mod multiply_long;
pub mod psr_transfer;
pub mod single_data_swap;
pub mod single_data_transfer;
pub mod software_interrupt;
pub mod undefined;

macro_rules! arm_instruction {
    ($name:ident) => {
        impl $name {
            pub fn new(value: u32) -> Self {
                Self { value }
            }

            #[inline]
            pub fn cond(&self) -> crate::Condition {
                use ironboyadvance_utils::bit::BitOps;
                self.value.bits(28..=31).into()
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    f,
                    "ArmInstruction: name: {:?}, bits: {} -> (0x{:08X})",
                    stringify!($name),
                    self.value,
                    self.value
                )
            }
        }
    };
}

pub(crate) use arm_instruction;
use ironboyadvance_utils::bit::BitOps;

pub type ArmInstructionFactory = fn(u32) -> ArmInstruction;

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
    //CoprocessorDataTransfer,
    //CoprocessorDataOperation,
    //CoprocessorRegisterTransfer,
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
        }
    }
}

impl Instruction for ArmInstruction {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match self {
            Self::DataProcessing(i) => i.execute(cpu),
            Self::PsrTransfer(i) => i.execute(cpu),
            Self::Multiply(i) => i.execute(cpu),
            Self::MultiplyLong(i) => i.execute(cpu),
            Self::SingleDataSwap(i) => i.execute(cpu),
            Self::BranchAndExchange(i) => i.execute(cpu),
            Self::HalfwordAndSignedDataTransfer(i) => i.execute(cpu),
            Self::SingleDataTransfer(i) => i.execute(cpu),
            Self::Undefined(i) => i.execute(cpu),
            Self::BlockDataTransfer(i) => i.execute(cpu),
            Self::BranchAndBranchWithLink(i) => i.execute(cpu),
            Self::SoftwareInterrupt(i) => i.execute(cpu),
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        match self {
            Self::DataProcessing(i) => i.disassemble(cpu),
            Self::PsrTransfer(i) => i.disassemble(cpu),
            Self::Multiply(i) => i.disassemble(cpu),
            Self::MultiplyLong(i) => i.disassemble(cpu),
            Self::SingleDataSwap(i) => i.disassemble(cpu),
            Self::BranchAndExchange(i) => i.disassemble(cpu),
            Self::HalfwordAndSignedDataTransfer(i) => i.disassemble(cpu),
            Self::SingleDataTransfer(i) => i.disassemble(cpu),
            Self::Undefined(i) => i.disassemble(cpu),
            Self::BlockDataTransfer(i) => i.disassemble(cpu),
            Self::BranchAndBranchWithLink(i) => i.disassemble(cpu),
            Self::SoftwareInterrupt(i) => i.disassemble(cpu),
        }
    }
}

impl fmt::Display for ArmInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataProcessing(i) => i.fmt(f),
            Self::PsrTransfer(i) => i.fmt(f),
            Self::Multiply(i) => i.fmt(f),
            Self::MultiplyLong(i) => i.fmt(f),
            Self::SingleDataSwap(i) => i.fmt(f),
            Self::BranchAndExchange(i) => i.fmt(f),
            Self::HalfwordAndSignedDataTransfer(i) => i.fmt(f),
            Self::SingleDataTransfer(i) => i.fmt(f),
            Self::Undefined(i) => i.fmt(f),
            Self::BlockDataTransfer(i) => i.fmt(f),
            Self::BranchAndBranchWithLink(i) => i.fmt(f),
            Self::SoftwareInterrupt(i) => i.fmt(f),
        }
    }
}

pub fn generate_arm_lut() -> [ArmInstructionFactory; 4096] {
    let mut arm_lut: [ArmInstructionFactory; 4096] = [|value| ArmInstruction::Undefined(Undefined::new(value)); 4096];
    for (i, factory) in arm_lut.iter_mut().enumerate() {
        *factory = decode_arm(((i as u32 & 0x0FF0) << 16) | ((i as u32 & 0x000F) << 4));
    }
    arm_lut
}

fn decode_arm(index: u32) -> ArmInstructionFactory {
    let pattern = index & 0x0FFFFFFF;
    let set_flags = pattern.bit(20);
    let opcode = pattern.bits(21..=24);
    let test_opcode = (0b1000..=0b1011).contains(&opcode);
    match pattern.bits(26..=27) {
        0b00 => {
            if pattern.bit(25) {
                match !set_flags && test_opcode {
                    true => |value| ArmInstruction::PsrTransfer(PsrTransfer::new(value)),
                    false => |value| ArmInstruction::DataProcessing(DataProcessing::new(value)),
                }
            } else if pattern & 0x0FF000F0 == 0x01200010 {
                |value| ArmInstruction::BranchAndExchange(BranchAndExchange::new(value))
            } else if pattern & 0x010000F0 == 0x00000090 {
                match pattern.bit(23) {
                    true => |value| ArmInstruction::MultiplyLong(MultiplyLong::new(value)),
                    false => |value| ArmInstruction::Multiply(Multiply::new(value)),
                }
            } else if pattern & 0x010000F0 == 0x01000090 {
                |value| ArmInstruction::SingleDataSwap(SingleDataSwap::new(value))
            } else if pattern & 0x000000F0 == 0x000000B0 || pattern & 0x000000D0 == 0x000000D0 {
                |value| ArmInstruction::HalfwordAndSignedDataTransfer(HalfwordAndSignedDataTransfer::new(value))
            } else {
                match !set_flags && test_opcode {
                    true => |value| ArmInstruction::PsrTransfer(PsrTransfer::new(value)),
                    false => |value| ArmInstruction::DataProcessing(DataProcessing::new(value)),
                }
            }
        }
        0b01 => match pattern & 0x02000010 == 0x02000010 {
            true => |value| ArmInstruction::Undefined(Undefined::new(value)),
            false => |value| ArmInstruction::SingleDataTransfer(SingleDataTransfer::new(value)),
        },
        0b10 => match pattern.bit(25) {
            true => |value| ArmInstruction::BranchAndBranchWithLink(BranchAndBranchWithLink::new(value)),
            false => |value| ArmInstruction::BlockDataTransfer(BlockDataTransfer::new(value)),
        },
        0b11 => match pattern.bit(25) {
            true => match pattern.bit(24) {
                true => |value| ArmInstruction::SoftwareInterrupt(SoftwareInterrupt::new(value)),
                false => |value| ArmInstruction::Undefined(Undefined::new(value)),
            },
            false => |value| ArmInstruction::Undefined(Undefined::new(value)),
        },
        _ => |value| ArmInstruction::Undefined(Undefined::new(value)),
    }
}
