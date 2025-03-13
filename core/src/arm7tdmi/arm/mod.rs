use super::{cpu::Instruction, Condition, Register};

pub mod disassembler;
pub mod execute;
// change this to Kind
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
    HalfwordDataTransferRegister,
    HalfwordDataTransferImmediate,
    TransferPsrToRegister,
    TransferRegisterToPsr,
    DataProcessing,
}

impl From<u32> for ArmInstructionFormat {
    fn from(instruction: u32) -> ArmInstructionFormat {
        use ArmInstructionFormat::*;
        // Decoding order matters
        if instruction & 0x0FFFFFF0 == 0x012FFF10 {
            BranchAndExchange
        } else if instruction & 0x0E000000 == 0x0800_0000 {
            BlockDataTransfer
        } else if instruction & 0x0F000000 == 0x0A000000 || instruction & 0x0F000000 == 0x0B000000 {
            BranchAndBranchWithLink
        } else if instruction & 0x0F000000 == 0x0F000000 {
            SoftwareInterrupt
        } else if instruction & 0x0E000010 == 0x06000010 {
            Undefined
        } else if instruction & 0x0C000000 == 0x04000000 {
            SingleDataTransfer
        } else if instruction & 0x0FB00FF0 == 0x01000090 {
            SingleDataSwap
        } else if instruction & 0x0F8000F0 == 0x00000090 {
            Multiply
        } else if instruction & 0x0F8000F0 == 0x00800090 {
            MultiplyLong
        } else if instruction & 0x0E400F90 == 0x00000090 {
            HalfwordDataTransferRegister
        } else if instruction & 0x0E400090 == 0x00400090 {
            HalfwordDataTransferImmediate
        } else if instruction & 0x0FBF0000 == 0x010F0000 {
            TransferPsrToRegister
        } else if instruction & 0x0DB0F000 == 0x0120F000 {
            TransferRegisterToPsr
        } else if instruction & 0x0C000000 == 0x0000_0000 {
            DataProcessing
        } else {
            unimplemented!("Instruction undefined or unimplemented")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmInstruction {
    format: ArmInstructionFormat,
    value: u32,
    executed_pc: u32,
}

impl Instruction for ArmInstruction {
    type Size = u32;

    fn decode(instruction: u32, executed_pc: u32) -> ArmInstruction {
        ArmInstruction {
            format: instruction.into(),
            value: instruction,
            executed_pc,
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

    pub fn offset(&self) -> i32 {
        (((self.value & 0xFFFFFF) << 8) as i32) >> 6
    }
}
