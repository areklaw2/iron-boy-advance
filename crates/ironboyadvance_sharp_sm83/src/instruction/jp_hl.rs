use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct JpHl;

impl JpHl {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for JpHl {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let hl = cpu.registers().hl();
        cpu.set_pc(hl);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "JP HL".to_string()
    }
}
