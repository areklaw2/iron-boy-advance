use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct OrAImm8;

impl OrAImm8 {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for OrAImm8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.fetch_byte();
        let result = cpu.registers().a() | value;
        cpu.registers_mut().set_a(result);

        cpu.registers_mut().f_mut().set_zero(result == 0);
        cpu.registers_mut().f_mut().set_subtraction(false);
        cpu.registers_mut().f_mut().set_half_carry(false);
        cpu.registers_mut().f_mut().set_carry(false);
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_byte();
        format!("OR A,{:#04X}", value)
    }
}
