use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LdSpHl;

impl LdSpHl {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for LdSpHl {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let hl = cpu.registers().hl();
        cpu.registers_mut().set_sp(hl);
        cpu.bus_mut().idle_cycle();
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "LD SP,HL".to_string()
    }
}
