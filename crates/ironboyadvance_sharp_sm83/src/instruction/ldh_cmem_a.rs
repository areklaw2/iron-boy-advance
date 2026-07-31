use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LdhCMemA;

impl LdhCMemA {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for LdhCMemA {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let address = 0xFF00 | cpu.registers().c() as u16;
        let value = cpu.registers().a();
        cpu.bus_mut().store_8(address, value);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "LD [FF00+C],A".to_string()
    }
}
