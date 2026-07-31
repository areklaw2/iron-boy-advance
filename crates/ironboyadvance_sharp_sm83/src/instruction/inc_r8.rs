use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct IncR8 {
    r8: Register8,
}

impl IncR8 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r8: Register8::from(opcode.bits(3..=5)),
        }
    }
}

impl Instruction for IncR8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_8(self.r8);
        let result = value.wrapping_add(1);
        cpu.set_register_8(self.r8, result);

        cpu.registers_mut().f_mut().set_zero(result == 0);
        cpu.registers_mut().f_mut().set_subtraction(false);
        cpu.registers_mut().f_mut().set_half_carry((value & 0x0F) + 1 > 0x0F);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("INC {}", self.r8)
    }
}
