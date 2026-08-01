use ironboyadvance_common::bits::BitOps;

use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;
use crate::{BitOpcode, Register8};

#[derive(Debug, Clone, Copy)]
pub(crate) struct BitOp {
    opcode: BitOpcode,
    r8: Register8,
    bit_index: u8,
}

impl BitOp {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            opcode: BitOpcode::from(opcode.bits(6..=7)),
            r8: Register8::from(opcode.bits(0..=2)),
            bit_index: opcode.bits(3..=5),
        }
    }
}

impl Instruction for BitOp {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_8(self.r8);
        match self.opcode {
            BitOpcode::BIT => {
                let is_clear = value & (1 << self.bit_index) == 0;

                cpu.registers_mut().f_mut().set_zero(is_clear);
                cpu.registers_mut().f_mut().set_subtraction(false);
                cpu.registers_mut().f_mut().set_half_carry(true);
            }
            BitOpcode::RES => cpu.set_register_8(self.r8, value & !(1 << self.bit_index)),
            BitOpcode::SET => cpu.set_register_8(self.r8, value | (1 << self.bit_index)),
        }
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("{} {},{}", self.opcode, self.bit_index, self.r8)
    }
}
