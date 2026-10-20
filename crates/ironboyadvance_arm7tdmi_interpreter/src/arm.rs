use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    arm::{
        ArmInstruction, BlockDataTransfer, BranchAndBranchWithLink, BranchAndExchange, DataProcessing,
        HalfwordAndSignedDataTransfer, Multiply, MultiplyLong, PsrTransfer, SingleDataSwap, SingleDataTransfer,
        SoftwareInterrupt, Undefined,
    },
    cpu::Arm7tdmiCpu,
    memory::MemoryInterface,
};
use ironboyadvance_common::bits::BitOps;

mod block_data_transfer;
mod branch_and_branch_with_link;
mod branch_and_exchange;
mod data_processing;
mod halfword_and_signed_data_transfer;
mod multiply;
mod multiply_long;
mod psr_transfer;
mod single_data_swap;
mod single_data_transfer;
mod software_interrupt;
mod undefined;

impl Execute for ArmInstruction {
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
            Self::CoprocessorDataOperation(i) => i.execute(cpu),
            Self::CoprocessorDataTransfer(i) => i.execute(cpu),
            Self::CoprocessorRegisterTransfer(i) => i.execute(cpu),
        }
    }
}

pub type ArmInstructionFactory = fn(u32) -> ArmInstruction;

pub(crate) fn generate_arm_lut() -> [ArmInstructionFactory; 4096] {
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
                false => match pattern.bit(4) {
                    true => |value| ArmInstruction::CoprocessorRegisterTransfer(Undefined::new(value)),
                    false => |value| ArmInstruction::CoprocessorDataOperation(Undefined::new(value)),
                },
            },
            false => |value| ArmInstruction::CoprocessorDataTransfer(Undefined::new(value)),
        },
        _ => |value| ArmInstruction::Undefined(Undefined::new(value)),
    }
}
