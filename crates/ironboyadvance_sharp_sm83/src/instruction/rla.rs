use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Rla;

impl Rla {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Rla {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let a = cpu.registers().a();
        let old_carry = cpu.registers_mut().f_mut().carry();
        let carry = a & 0x80 == 0x80;
        let result = (a << 1) | (if old_carry { 1 } else { 0 });

        cpu.registers_mut().f_mut().set_zero(false);
        cpu.registers_mut().f_mut().set_subtraction(false);
        cpu.registers_mut().f_mut().set_half_carry(false);
        cpu.registers_mut().f_mut().set_carry(carry);
        cpu.registers_mut().set_a(result);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "RLA".to_string()
    }
}
