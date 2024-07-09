#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArmInstructionFormat {
    BranchAndExchange,
    BranchAndBranchWithLink,
    Undefined,
    DataProcessing,
    BlockDataTransfer,
    SoftwareInterrupt,
    SingleDataTransfer,
    SingleDataSwap,
    Multiply,
    MultiplyLong,
    HalfDataTransferRegisterOffset,
    HalfDataTransferImmediateOffset,
    PsrTransfer,
}

impl From<u32> for ArmInstructionFormat {
    fn from(instruction: u32) -> ArmInstructionFormat {
        use ArmInstructionFormat::*;
        if instruction & 0x0F00_0000 == 0x0A00_0000 || instruction & 0x0F00_0000 == 0x0B00_0000 {
            return BranchAndBranchWithLink;
        }
        if instruction & 0x0FFF_FFF0 == 0x012F_FF10 {
            return BranchAndExchange;
        }
        if instruction & 0x0E00_0010 == 0x0600_0010 {
            return Undefined;
        }
        if instruction & 0x0C00_0000 == 0x0000_0000 {
            return DataProcessing;
        }
        if instruction & 0x0FC0_00F0 == 0x0000_0090 {
            return Multiply;
        }
        if instruction & 0x0F80_00F0 == 0x0080_0090 {
            return MultiplyLong;
        }
        if instruction & 0b0000_1111_1011_0000_0000_0000_0000_0000 == {
            return PsrTransfer;
        }

        return Undefined;
    }
}

impl ArmInstructionFormat {
    fn is_block_data_transfer(&self, instruction: u32) -> bool {
        instruction & 0x0E00_0000 == 0x0800_0000
    }

    fn is_software_interrupt(&self, instruction: u32) -> bool {
        instruction & 0x0F00_0000 == 0x0F00_0000
    }

    fn is_single_data_transfer(&self, instruction: u32) -> bool {
        instruction & 0x0C00_0000 == 0x0400_0000
    }

    fn is_single_data_swap(&self, instruction: u32) -> bool {
        instruction & 0x0F80_0FF0 == 0x0100_0090
    }
}
