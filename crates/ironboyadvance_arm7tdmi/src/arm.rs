use crate::arm::{
    block_data_transfer::BlockDataTransfer, branch_and_branch_link::BranchAndBranchWithLink,
    branch_and_exchange::BranchAndExchange, data_processing::DataProcessing,
    halfword_and_signed_data_transfer::HalfwordAndSignedDataTransfer, multiply::Multiply, multiply_long::MultiplyLong,
    psr_transfer::PsrTransfer, single_data_swap::SingleDataSwap, single_data_transfer::SingleDataTransfer,
    software_interrupt::SoftwareInterrupt, undefined::Undefined,
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

pub mod lut;

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
