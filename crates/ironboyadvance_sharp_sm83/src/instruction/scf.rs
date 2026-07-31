use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Scf;

impl Scf {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Scf {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        cpu.registers_mut().f_mut().set_carry(true);
        cpu.registers_mut().f_mut().set_half_carry(false);
        cpu.registers_mut().f_mut().set_subtraction(false);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "SCF".to_string()
    }
}
