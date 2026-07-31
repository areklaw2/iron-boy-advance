use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Res {
    r8: Register8,
    bit_index: u8,
}

impl Res {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r8: Register8::from(opcode.bits(0..=2)),
            bit_index: opcode.bits(3..=5),
        }
    }
}

impl Instruction for Res {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_8(self.r8);
        cpu.set_register_8(self.r8, value & !(1 << self.bit_index));
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("RES {},{}", self.bit_index, self.r8)
    }
}
