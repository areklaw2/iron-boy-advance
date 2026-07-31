use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LdhAImm8Mem;

impl LdhAImm8Mem {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for LdhAImm8Mem {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let address = 0xFF00 | cpu.fetch_byte() as u16;
        let value = cpu.bus().load_8(address);
        cpu.registers_mut().set_a(value);
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_byte();
        format!("LD A,[FF00+{:#04X}]", value)
    }
}
