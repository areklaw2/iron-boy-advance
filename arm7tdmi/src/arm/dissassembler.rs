#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArmInstructionFormat {
    BranchAndExchange,
    BranchAndBranchWithLink,
    Undefined,
    DataProcessing,
    Multiply,
    MultiplyLong,
    TransferPsrToRegister,
    TransferRegisterToPsr,
    TransferRegisterOrImmediateValueToPsrFlags,
    SingleDataTransfer,
    HalfwordDataTransferRegisterOffset,
    HalfwordDataTransferImmediateOffset,
    BlockDataTransfer,
    SoftwareInterrupt,
    SingleDataSwap,
}

impl From<u32> for ArmInstructionFormat {
    fn from(instruction: u32) -> ArmInstructionFormat {
        use ArmInstructionFormat::*;
        if instruction & 0x0E00_0000 == 0x0A00_0000 {
            return BranchAndBranchWithLink;
        }
        if instruction & 0x0FFF_FFF0 == 0x012F_FF10 {
            return BranchAndExchange;
        }
        if instruction & 0x0F00_0000 == 0x0F00_0000 {
            return SoftwareInterrupt;
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
        if instruction & 0x0FBF_0FFF == 0x010F_0000 {
            return TransferPsrToRegister;
        }
        if instruction & 0x0FBF_FFF0 == 0x0129_F000 {
            return TransferRegisterToPsr;
        }
        if instruction & 0x0DBF_F000 == 0x0128_F000 {
            return TransferRegisterOrImmediateValueToPsrFlags;
        }
        if instruction & 0x0C00_0000 == 0x0400_0000 {
            return SingleDataTransfer;
        }
        if instruction & 0x0E40_0F90 == 0x0000_0090 {
            return HalfwordDataTransferRegisterOffset;
        }
        if instruction & 0x0E40_0090 == 0x0040_0090 {
            return HalfwordDataTransferImmediateOffset;
        }
        if instruction & 0x0E00_0000 == 0x0800_0000 {
            return BlockDataTransfer;
        }
        if instruction & 0x0FB0_0FF0 == 0x0100_0090 {
            return SingleDataSwap;
        }
        return Undefined;
    }
}
