use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LdImm16Sp;

impl LdImm16Sp {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for LdImm16Sp {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let address = cpu.fetch_word();
        let sp = cpu.registers().sp();
        cpu.bus_mut().store_16(address, sp);
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_word();
        format!("LD {:#04X},SP", value)
    }
}
