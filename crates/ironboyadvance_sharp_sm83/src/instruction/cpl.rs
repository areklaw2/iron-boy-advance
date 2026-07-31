use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Cpl;

impl Cpl {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Cpl {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let not_a = !cpu.registers().a();
        cpu.registers_mut().set_a(not_a);
        cpu.registers_mut().f_mut().set_subtraction(true);
        cpu.registers_mut().f_mut().set_half_carry(true);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "CPL".to_string()
    }
}
