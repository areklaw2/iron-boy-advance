use crate::memory::MemoryInterface;

use super::{
    cpu::{Arm7tdmiCpu, Instruction},
    Condition, Register,
};

pub mod disassembler;
pub mod execute;

const BX_FORMAT: u32 = 0x012FFF10;
const BX_MASK: u32 = 0x0FFFFFF0;
const B_BL_FORMAT: u32 = 0x0A000000;
const B_BL_MASK: u32 = 0x0E000000;

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
        //TODO: Make constants for masks and formats
        if instruction & BX_MASK == BX_FORMAT {
            BranchAndExchange
        } else if instruction & B_BL_MASK == B_BL_FORMAT {
            BranchAndBranchWithLink
        } else if instruction & 0x0F00_0000 == 0x0F00_0000 {
            SoftwareInterrupt
        } else if instruction & 0x0E00_0010 == 0x0600_0010 {
            Undefined
        } else if instruction & 0x0C00_0000 == 0x0000_0000 {
            DataProcessing
        } else if instruction & 0x0FC0_00F0 == 0x0000_0090 {
            Multiply
        } else if instruction & 0x0F80_00F0 == 0x0080_0090 {
            MultiplyLong
        } else if instruction & 0x0FBF_0FFF == 0x010F_0000 {
            TransferPsrToRegister
        } else if instruction & 0x0FBF_FFF0 == 0x0129_F000 {
            TransferRegisterToPsr
        } else if instruction & 0x0DBF_F000 == 0x0128_F000 {
            TransferRegisterOrImmediateValueToPsrFlags
        } else if instruction & 0x0C00_0000 == 0x0400_0000 {
            SingleDataTransfer
        } else if instruction & 0x0E40_0F90 == 0x0000_0090 {
            HalfwordDataTransferRegisterOffset
        } else if instruction & 0x0E40_0090 == 0x0040_0090 {
            HalfwordDataTransferImmediateOffset
        } else if instruction & 0x0E00_0000 == 0x0800_0000 {
            BlockDataTransfer
        } else if instruction & 0x0FB0_0FF0 == 0x0100_0090 {
            SingleDataSwap
        } else {
            Undefined
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmInstruction {
    format: ArmInstructionFormat,
    value: u32,
}

impl Instruction for ArmInstruction {
    type Size = u32;

    fn decode(instruction: u32) -> ArmInstruction {
        ArmInstruction {
            format: instruction.into(),
            value: instruction,
        }
    }

    fn disassamble(&self) -> String {
        use ArmInstructionFormat::*;
        match self.format {
            BranchAndExchange => self.disassemble_branch_and_exchange(),
            BranchAndBranchWithLink => self.disassemble_branch_and_branch_with_link(),
            _ => todo!(),
        }
    }

    fn value(&self) -> u32 {
        self.value
    }
}

impl ArmInstruction {
    pub fn new(format: ArmInstructionFormat, instruction: u32) -> ArmInstruction {
        ArmInstruction {
            format,
            value: instruction,
        }
    }

    pub fn link(&self) -> bool {
        self.value & (1 << 24) != 0
    }

    pub fn cond(&self) -> Condition {
        (self.value >> 28 & 0xF).into()
    }

    pub fn rn(&self) -> Register {
        use ArmInstructionFormat::*;
        match self.format {
            BranchAndExchange => (self.value & 0xF).into(),
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::arm7tdmi::{Condition, Register};

    use super::{ArmInstruction, ArmInstructionFormat};
    use ArmInstructionFormat::*;

    #[test]
    fn get_condition() {
        let instruction = ArmInstruction::new(BranchAndExchange, 0);
        assert_eq!(instruction.cond(), Condition::HI)
    }

    #[test]
    fn get_rn() {
        let instruction = ArmInstruction::new(BranchAndExchange, 0);
        assert_eq!(instruction.rn(), Register::R12)
    }
}
