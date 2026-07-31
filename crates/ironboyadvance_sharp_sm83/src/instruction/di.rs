use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Di;

impl Di {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Di {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        cpu.set_disable_interrupt_delay(2);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "DI".to_string()
    }
}
