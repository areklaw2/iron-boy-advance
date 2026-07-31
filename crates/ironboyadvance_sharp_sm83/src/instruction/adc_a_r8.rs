use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct AdcAR8 {
    r8: Register8,
}

impl AdcAR8 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r8: Register8::from(opcode.bits(0..=2)),
        }
    }
}

impl Instruction for AdcAR8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value1 = cpu.registers().a();
        let value2 = cpu.register_8(self.r8);
        let carry = if cpu.registers().f().carry() { 1 } else { 0 };
        let result = value1.wrapping_add(value2).wrapping_add(carry);
        cpu.registers_mut().set_a(result);

        cpu.registers_mut().f_mut().set_zero(result == 0);
        cpu.registers_mut().f_mut().set_subtraction(false);
        cpu.registers_mut()
            .f_mut()
            .set_half_carry((value1 & 0x0F) + (value2 & 0x0F) + carry > 0x0F);
        cpu.registers_mut()
            .f_mut()
            .set_carry(value1 as u16 + value2 as u16 + carry as u16 > 0xFF);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("ADC A,{}", self.r8)
    }
}
