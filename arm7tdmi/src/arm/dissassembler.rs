#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArmInstructionFormat {
    BranchAndExchange,
    BlockDataTransfer,
    BranchAndBranchWithLink,
    SoftwareInterrupt,
    Undefined,
    SingleDataTransfer,
    SingleDataSwap,
    Multiply,
    MultiplyLong,
    HalfDataTransferRegisterOffset,
    HalfDataTransferImmediateOffset,
    PSRTransfer,
    DataProcessing,
}

impl From<u32> for ArmInstructionFormat {
    fn from(instruction: u32) -> ArmInstructionFormat {
        match instruction {
            _ => Self::Undefined,
        }
    }
}

impl ArmInstructionFormat {
    fn is_branch_and_exchange(&self, instruction: u32) -> bool {
        instruction & 0x0FFF_FFF0 == 0x012F_FF10
    }

    fn is_block_data_transfer(&self, instruction: u32) -> bool {
        instruction & 0x0E00_0000 == 0x0800_0000
    }

    fn is_branch_and_branch_with_link(&self, instruction: u32) -> bool {
        instruction & 0x0F00_0000 == 0x0A00_0000 || instruction & 0x0F00_0000 == 0x0B00_0000
    }

    fn is_software_interrupt(&self, instruction: u32) -> bool {
        instruction & 0x0F00_0000 == 0x0F00_0000
    }

    fn is_undefined(&self, instruction: u32) -> bool {
        instruction & 0x0E00_0010 == 0x0600_0010
    }

    fn is_single_data_transfer(&self, instruction: u32) -> bool {
        instruction & 0x0C00_0000 == 0x0400_0000
    }

    fn is_single_data_swap(&self, instruction: u32) -> bool {
        instruction & 0x0F80_0FF0 == 0x0100_0090
    }

    fn is_multiply(&self, instruction: u32) -> bool {
        instruction & 0x0F80_00F0 == 0x0000_0090
    }

    fn is_multiply_long(&self, instruction: u32) -> bool {
        instruction & 0x0F80_00F0 == 0x800090
    }
}
