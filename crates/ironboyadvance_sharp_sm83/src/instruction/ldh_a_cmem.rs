use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LdhACMem;

impl LdhACMem {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for LdhACMem {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let address = 0xFF00 | cpu.registers().c() as u16;
        let value = cpu.bus().load_8(address);
        cpu.registers_mut().set_a(value);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "LD A,[FF00+C]".to_string()
    }
}
