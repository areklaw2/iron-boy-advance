use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SubAImm8;

impl SubAImm8 {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for SubAImm8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value1 = cpu.registers().a();
        let value2 = cpu.fetch_byte();
        let result = value1.wrapping_sub(value2);
        cpu.registers_mut().set_a(result);

        cpu.registers_mut().f_mut().set_zero(result == 0);
        cpu.registers_mut().f_mut().set_subtraction(true);
        cpu.registers_mut().f_mut().set_half_carry((value1 & 0x0F) < (value2 & 0x0F));
        cpu.registers_mut().f_mut().set_carry((value1 as u16) < (value2 as u16));
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_byte();
        format!("SUB A,{:#04X}", value)
    }
}
