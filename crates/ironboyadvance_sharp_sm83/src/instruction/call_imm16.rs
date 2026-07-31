use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CallImm16;

impl CallImm16 {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for CallImm16 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let return_address = cpu.pc() + 2;
        cpu.push_stack(return_address);
        let word = cpu.fetch_word();
        cpu.set_pc(word);
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_word();
        format!("CALL {:#04X}", value)
    }
}
