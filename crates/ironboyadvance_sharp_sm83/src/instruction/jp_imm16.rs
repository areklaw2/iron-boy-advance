use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct JpImm16;

impl JpImm16 {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for JpImm16 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let word = cpu.fetch_word();
        cpu.set_pc(word);
        cpu.bus_mut().idle_cycle();
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_word();
        format!("JP {:#04X}", value)
    }
}
