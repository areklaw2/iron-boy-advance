use ironboyadvance_common::bits::BitOps;

use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InterruptEnable {
    enables_interrupts: bool,
}

impl InterruptEnable {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            enables_interrupts: opcode.bit(3),
        }
    }
}

impl Instruction for InterruptEnable {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        match self.enables_interrupts {
            true => cpu.set_enable_interrupt_delay(2),
            false => cpu.set_disable_interrupt_delay(2),
        };
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) -> String {
        match self.enables_interrupts {
            true => "EI".to_string(),
            false => "DI".to_string(),
        }
    }
}
