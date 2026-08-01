use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct IncrementDecrement {
    r8: Register8,
    is_decrement: bool,
}

impl IncrementDecrement {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r8: Register8::from(opcode.bits(3..=5)),
            is_decrement: opcode.bit(0),
        }
    }
}

impl Instruction for IncrementDecrement {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        let value = cpu.register_8(self.r8);
        let result = match self.is_decrement {
            true => value.wrapping_sub(1),
            false => value.wrapping_add(1),
        };
        cpu.set_register_8(self.r8, result);

        cpu.registers_mut().f_mut().set_zero(result == 0);
        cpu.registers_mut().f_mut().set_subtraction(self.is_decrement);
        cpu.registers_mut().f_mut().set_half_carry(match self.is_decrement {
            true => (value & 0x0F) == 0,
            false => (value & 0x0F) + 1 > 0x0F,
        });
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) -> String {
        match self.is_decrement {
            true => format!("DEC {}", self.r8),
            false => format!("INC {}", self.r8),
        }
    }
}
