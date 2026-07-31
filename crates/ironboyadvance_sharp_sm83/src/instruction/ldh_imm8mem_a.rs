use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LdhImm8MemA;

impl LdhImm8MemA {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for LdhImm8MemA {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let address = 0xFF00 | cpu.fetch_byte() as u16;
        let value = cpu.registers().a();
        cpu.bus_mut().store_8(address, value);
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_byte();
        format!("LD [FF00+{:#04X}],A", value)
    }
}
