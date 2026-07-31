use crate::cpu::SharpSm83;
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
    fn execute<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) {}

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "Undefined".into()
    }
}
