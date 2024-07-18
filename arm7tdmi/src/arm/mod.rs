use dissassembler::ArmInstructionFormat;

use crate::Cpu;

pub mod dissassembler;
pub mod execute;

impl Cpu {
    pub fn arm_decode(&self, instruction: u32) -> ArmInstructionFormat {
        // this will probably turn into its on struct or something i'm thinking
        // maybe i could get convoluted and make decoding functions for each instruction format
        // maybe some type of trait to with functions
        // struct just need to start decoding first
        todo!("{}", instruction)
    }

    pub fn arm_execute(&self, instruction_format: ArmInstructionFormat, instruction: u32) {
        todo!("{:?} {}", instruction_format, instruction)
    }
}
