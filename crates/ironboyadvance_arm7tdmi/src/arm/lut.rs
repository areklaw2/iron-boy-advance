use ArmInstruction::*;
use ironboyadvance_utils::bit::BitOps;

use crate::arm::{
    block_data_transfer::BlockDataTransfer, branch_and_branch_link::BranchAndBranchWithLink,
    branch_and_exchange::BranchAndExchange, data_processing::DataProcessing,
    halfword_and_signed_data_transfer::HalfwordAndSignedDataTransfer, multiply::Multiply, multiply_long::MultiplyLong,
    psr_transfer::PsrTransfer, single_data_swap::SingleDataSwap, single_data_transfer::SingleDataTransfer,
    software_interrupt::SoftwareInterrupt, undefined::Undefined,
};

use super::ArmInstruction;

pub fn generate_arm_lut() -> [ArmInstruction; 4096] {
    let mut arm_lut = [Undefined(Undefined::default()); 4096];
    for (i, arm_instruction_kind) in arm_lut.iter_mut().enumerate() {
        *arm_instruction_kind = decode_arm(((i as u32 & 0x0FF0) << 16) | ((i as u32 & 0x000F) << 4));
    }
    arm_lut
}

fn decode_arm(instruction: u32) -> ArmInstruction {
    let pattern = instruction & 0x0FFFFFFF;
    let set_flags = pattern.bit(20);
    let opcode = pattern.bits(21..=24);
    let test_opcode = (0b1000..=0b1011).contains(&opcode);
    match pattern.bits(26..=27) {
        0b00 => {
            if pattern.bit(25) {
                match !set_flags && test_opcode {
                    true => PsrTransfer(PsrTransfer::new(instruction)),
                    false => DataProcessing(DataProcessing::new(instruction)),
                }
            } else if pattern & 0x0FF000F0 == 0x01200010 {
                BranchAndExchange(BranchAndExchange::new(instruction))
            } else if pattern & 0x010000F0 == 0x00000090 {
                match pattern.bit(23) {
                    true => MultiplyLong(MultiplyLong::new(instruction)),
                    false => Multiply(Multiply::new(instruction)),
                }
            } else if pattern & 0x010000F0 == 0x01000090 {
                SingleDataSwap(SingleDataSwap::new(instruction))
            } else if pattern & 0x000000F0 == 0x000000B0 || pattern & 0x000000D0 == 0x000000D0 {
                HalfwordAndSignedDataTransfer(HalfwordAndSignedDataTransfer::new(instruction))
            } else {
                match !set_flags && test_opcode {
                    true => PsrTransfer(PsrTransfer::new(instruction)),
                    false => DataProcessing(DataProcessing::new(instruction)),
                }
            }
        }
        0b01 => match pattern & 0x02000010 == 0x02000010 {
            true => Undefined(Undefined::new(instruction)),
            false => SingleDataTransfer(SingleDataTransfer::new(instruction)),
        },
        0b10 => match pattern.bit(25) {
            true => BranchAndBranchWithLink(BranchAndBranchWithLink::new(instruction)),
            false => BlockDataTransfer(BlockDataTransfer::new(instruction)),
        },
        0b11 => match pattern.bit(25) {
            true => match pattern.bit(24) {
                true => SoftwareInterrupt(SoftwareInterrupt::new(instruction)),
                //CoprocessorDataOperation
                //CoprocessorRegisterTransfer
                false => Undefined(Undefined::new(instruction)),
            },
            //CoprocessorDataTransfer
            false => Undefined(Undefined::new(instruction)),
        },
        _ => Undefined(Undefined::new(instruction)),
    }
}
