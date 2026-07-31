use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::instruction::cb::set_rotate_shift_flags;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Rl {
    r8: Register8,
}

impl Rl {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r8: Register8::from(opcode.bits(0..=2)),
        }
    }
}

impl Instruction for Rl {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_8(self.r8);
        let carry = value & 0x80 == 0x80;
        let old_carry = cpu.registers().f().carry();
        let result = (value << 1) | (if old_carry { 1 } else { 0 });
        cpu.set_register_8(self.r8, result);
        set_rotate_shift_flags(cpu, result, carry);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("RL {}", self.r8)
    }
}
