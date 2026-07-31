use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LdR8R8 {
    dest: Register8,
    src: Register8,
}

impl LdR8R8 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            dest: Register8::from(opcode.bits(3..=5)),
            src: Register8::from(opcode.bits(0..=2)),
        }
    }
}

impl Instruction for LdR8R8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_8(self.src);
        cpu.set_register_8(self.dest, value);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("LD {},{}", self.dest, self.src)
    }
}
