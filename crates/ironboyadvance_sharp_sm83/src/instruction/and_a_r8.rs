use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct AndAR8 {
    r8: Register8,
}

impl AndAR8 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r8: Register8::from(opcode.bits(0..=2)),
        }
    }
}

impl Instruction for AndAR8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_8(self.r8);
        let result = cpu.registers().a() & value;
        cpu.registers_mut().set_a(result);

        cpu.registers_mut().f_mut().set_zero(result == 0);
        cpu.registers_mut().f_mut().set_subtraction(false);
        cpu.registers_mut().f_mut().set_half_carry(true);
        cpu.registers_mut().f_mut().set_carry(false);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("AND A,{}", self.r8)
    }
}
