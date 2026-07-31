use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Ccf;

impl Ccf {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Ccf {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let carry = !cpu.registers_mut().f_mut().carry();
        cpu.registers_mut().f_mut().set_carry(carry);
        cpu.registers_mut().f_mut().set_half_carry(false);
        cpu.registers_mut().f_mut().set_subtraction(false);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "CCF".to_string()
    }
}
