use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Undefined;

impl Undefined {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Undefined {
    fn execute<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) {}

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) -> String {
        "Undefined".into()
    }
}
