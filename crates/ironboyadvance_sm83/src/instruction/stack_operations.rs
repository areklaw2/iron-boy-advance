use ironboyadvance_common::bits::BitOps;

use crate::Register16Stack;
use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct StackOperations {
    r16_stack: Register16Stack,
    is_push: bool,
}

impl StackOperations {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r16_stack: Register16Stack::from(opcode.bits(4..=5)),
            is_push: opcode.bit(2),
        }
    }
}

impl Instruction for StackOperations {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        match self.is_push {
            true => {
                let value = cpu.register_16_stack(self.r16_stack);
                cpu.push_stack(value);
            }
            false => {
                let value = cpu.pop_stack();
                cpu.set_register_16_stack(self.r16_stack, value);
            }
        }
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) -> String {
        match self.is_push {
            true => format!("PUSH {}", self.r16_stack),
            false => format!("POP {}", self.r16_stack),
        }
    }
}
