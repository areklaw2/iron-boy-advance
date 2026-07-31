use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LdR8Imm8 {
    r8: Register8,
}

impl LdR8Imm8 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r8: Register8::from(opcode.bits(3..=5)),
        }
    }
}

impl Instruction for LdR8Imm8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.fetch_byte();
        cpu.set_register_8(self.r8, value);
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_byte();
        format!("LD {},{:#04X}", self.r8, value)
    }
}
