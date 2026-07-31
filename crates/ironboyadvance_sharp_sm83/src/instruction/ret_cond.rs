use ironboyadvance_common::bits::BitOps;

use crate::Condition;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RetCond {
    cond: Condition,
}

impl RetCond {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            cond: Condition::from(opcode.bits(3..=4)),
        }
    }
}

impl Instruction for RetCond {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        if cpu.is_condition_met(self.cond) {
            let value = cpu.pop_stack();
            cpu.set_pc(value);
            cpu.bus_mut().idle_cycle();
        }
        cpu.bus_mut().idle_cycle();
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("RET {}", self.cond)
    }
}
