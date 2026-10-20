use crate::{Dissasemble, arm::*, cpu::Arm7tdmiCpu, memory::MemoryInterface};

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

impl Dissasemble for ArmInstruction {
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
            Self::CoprocessorDataOperation(i) => i.disassemble(cpu),
            Self::CoprocessorDataTransfer(i) => i.disassemble(cpu),
            Self::CoprocessorRegisterTransfer(i) => i.disassemble(cpu),
        }
    }
}
