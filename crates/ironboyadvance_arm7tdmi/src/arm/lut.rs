use ArmInstructionKind::*;
use ironboyadvance_utils::bit::BitOps;

use super::ArmInstructionKind;

pub fn generate_arm_lut() -> [ArmInstructionKind; 4096] {
    let mut arm_lut = [Undefined; 4096];
    for (i, arm_instruction_kind) in arm_lut.iter_mut().enumerate() {
        *arm_instruction_kind = decode_arm(((i as u32 & 0x0FF0) << 16) | ((i as u32 & 0x000F) << 4));
    }
    arm_lut
}

fn decode_arm(instruction: u32) -> ArmInstructionKind {
    let pattern = instruction & 0x0FFFFFFF;
    let set_flags = pattern.bit(20);
    let opcode = pattern.bits(21..=24);
    let test_opcode = (0b1000..=0b1011).contains(&opcode);
    match pattern.bits(26..=27) {
        0b00 => {
            if pattern.bit(25) {
                match !set_flags && test_opcode {
                    true => PsrTransfer,
                    false => DataProcessing,
                }
            } else if pattern & 0x0FF000F0 == 0x01200010 {
                BranchAndExchange
            } else if pattern & 0x010000F0 == 0x00000090 {
                match pattern.bit(23) {
                    true => MultiplyLong,
                    false => Multiply,
                }
            } else if pattern & 0x010000F0 == 0x01000090 {
                SingleDataSwap
            } else if pattern & 0x000000F0 == 0x000000B0 || pattern & 0x000000D0 == 0x000000D0 {
                HalfwordAndSignedDataTransfer
            } else {
                match !set_flags && test_opcode {
                    true => PsrTransfer,
                    false => DataProcessing,
                }
            }
        }
        0b01 => match pattern & 0x02000010 == 0x02000010 {
            true => Undefined,
            false => SingleDataTransfer,
        },
        0b10 => match pattern.bit(25) {
            true => BranchAndBranchWithLink,
            false => BlockDataTransfer,
        },
        0b11 => match pattern.bit(25) {
            true => match pattern.bit(24) {
                true => SoftwareInterrupt,
                //CoprocessorDataOperation
                //CoprocessorRegisterTransfer
                false => Undefined,
            },
            //CoprocessorDataTransfer
            false => Undefined,
        },
        _ => Undefined,
    }
}
