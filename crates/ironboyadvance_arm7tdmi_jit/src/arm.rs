use crate::Compile;
use dynasmrt::{Assembler, aarch64::Aarch64Relocation};
use ironboyadvance_arm7tdmi::{
    arm::{
        ArmInstruction, BlockDataTransfer, BranchAndBranchWithLink, BranchAndExchange, DataProcessing,
        HalfwordAndSignedDataTransfer, Multiply, MultiplyLong, PsrTransfer, SingleDataSwap, SingleDataTransfer,
        SoftwareInterrupt, Undefined,
    },
    memory::MemoryInterface,
};
use ironboyadvance_common::bits::BitOps;

mod branch_and_branch_with_link;
mod branch_and_exchange;
mod data_processing;
mod software_interrupt;
mod undefined;

impl Compile for ArmInstruction {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        match self {
            Self::DataProcessing(i) => i.compile::<I>(assembler),
            Self::PsrTransfer(_i) => todo!(),
            Self::Multiply(_i) => todo!(),
            Self::MultiplyLong(_i) => todo!(),
            Self::SingleDataSwap(_i) => todo!(),
            Self::BranchAndExchange(i) => i.compile::<I>(assembler),
            Self::HalfwordAndSignedDataTransfer(_i) => todo!(),
            Self::SingleDataTransfer(_i) => todo!(),
            Self::Undefined(i) => i.compile::<I>(assembler),
            Self::BlockDataTransfer(_i) => todo!(),
            Self::BranchAndBranchWithLink(i) => i.compile::<I>(assembler),
            Self::SoftwareInterrupt(i) => i.compile::<I>(assembler),
            Self::CoprocessorDataOperation(i) => i.compile::<I>(assembler),
            Self::CoprocessorDataTransfer(i) => i.compile::<I>(assembler),
            Self::CoprocessorRegisterTransfer(i) => i.compile::<I>(assembler),
        }
    }
}

pub fn decode_arm(value: u32) -> ArmInstruction {
    let pattern = value & 0x0FFFFFFF;
    let set_flags = pattern.bit(20);
    let opcode = pattern.bits(21..=24);
    let test_opcode = (0b1000..=0b1011).contains(&opcode);
    match pattern.bits(26..=27) {
        0b00 => {
            if pattern.bit(25) {
                match !set_flags && test_opcode {
                    true => ArmInstruction::PsrTransfer(PsrTransfer::new(value)),
                    false => ArmInstruction::DataProcessing(DataProcessing::new(value)),
                }
            } else if pattern & 0x0FF000F0 == 0x01200010 {
                ArmInstruction::BranchAndExchange(BranchAndExchange::new(value))
            } else if pattern & 0x010000F0 == 0x00000090 {
                match pattern.bit(23) {
                    true => ArmInstruction::MultiplyLong(MultiplyLong::new(value)),
                    false => ArmInstruction::Multiply(Multiply::new(value)),
                }
            } else if pattern & 0x010000F0 == 0x01000090 {
                ArmInstruction::SingleDataSwap(SingleDataSwap::new(value))
            } else if pattern & 0x000000F0 == 0x000000B0 || pattern & 0x000000D0 == 0x000000D0 {
                ArmInstruction::HalfwordAndSignedDataTransfer(HalfwordAndSignedDataTransfer::new(value))
            } else {
                match !set_flags && test_opcode {
                    true => ArmInstruction::PsrTransfer(PsrTransfer::new(value)),
                    false => ArmInstruction::DataProcessing(DataProcessing::new(value)),
                }
            }
        }
        0b01 => match pattern & 0x02000010 == 0x02000010 {
            true => ArmInstruction::Undefined(Undefined::new(value)),
            false => ArmInstruction::SingleDataTransfer(SingleDataTransfer::new(value)),
        },
        0b10 => match pattern.bit(25) {
            true => ArmInstruction::BranchAndBranchWithLink(BranchAndBranchWithLink::new(value)),
            false => ArmInstruction::BlockDataTransfer(BlockDataTransfer::new(value)),
        },
        0b11 => match pattern.bit(25) {
            true => match pattern.bit(24) {
                true => ArmInstruction::SoftwareInterrupt(SoftwareInterrupt::new(value)),
                false => match pattern.bit(4) {
                    true => ArmInstruction::CoprocessorRegisterTransfer(Undefined::new(value)),
                    false => ArmInstruction::CoprocessorDataOperation(Undefined::new(value)),
                },
            },
            false => ArmInstruction::CoprocessorDataTransfer(Undefined::new(value)),
        },
        _ => ArmInstruction::Undefined(Undefined::new(value)),
    }
}
