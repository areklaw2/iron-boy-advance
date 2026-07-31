use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::instruction::cb::set_rotate_shift_flags;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Rr {
    r8: Register8,
}

impl Rr {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r8: Register8::from(opcode.bits(0..=2)),
        }
    }
}

impl Instruction for Rr {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_8(self.r8);
        let carry = value & 0x01 == 0x01;
        let old_carry = cpu.registers().f().carry();
        let result = (value >> 1) | (if old_carry { 0x80 } else { 0 });
        cpu.set_register_8(self.r8, result);
        set_rotate_shift_flags(cpu, result, carry);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("RR {}", self.r8)
    }
}
