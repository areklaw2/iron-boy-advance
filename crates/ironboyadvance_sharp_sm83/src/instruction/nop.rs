use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Nop;

impl Nop {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Nop {
    fn execute<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) {}

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) -> String {
        "NOP".to_string()
    }
}
