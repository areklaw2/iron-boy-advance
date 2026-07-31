use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::instruction::cb::set_rotate_shift_flags;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Sra {
    r8: Register8,
}

impl Sra {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r8: Register8::from(opcode.bits(0..=2)),
        }
    }
}

impl Instruction for Sra {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_8(self.r8);
        let carry = value & 0x01 == 0x01;
        let result = (value >> 1) | (value & 0x80);
        cpu.set_register_8(self.r8, result);
        set_rotate_shift_flags(cpu, result, carry);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("SRA {}", self.r8)
    }
}
