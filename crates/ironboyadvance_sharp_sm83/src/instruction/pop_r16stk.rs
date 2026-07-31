use ironboyadvance_common::bits::BitOps;

use crate::Register16Stack;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PopR16Stk {
    r16stk: Register16Stack,
}

impl PopR16Stk {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r16stk: Register16Stack::from(opcode.bits(4..=5)),
        }
    }
}

impl Instruction for PopR16Stk {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.pop_stack();
        cpu.set_register_16_stack(self.r16stk, value);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("POP {}", self.r16stk)
    }
}
