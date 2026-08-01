use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Halt;

impl Halt {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Halt {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        cpu.set_halted(true);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) -> String {
        "HALT".to_string()
    }
}
