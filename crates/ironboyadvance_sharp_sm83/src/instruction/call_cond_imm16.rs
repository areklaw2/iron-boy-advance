use ironboyadvance_common::bits::BitOps;

use crate::Condition;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CallCondImm16 {
    cond: Condition,
}

impl CallCondImm16 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            cond: Condition::from(opcode.bits(3..=4)),
        }
    }
}

impl Instruction for CallCondImm16 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let word = cpu.fetch_word();
        if cpu.is_condition_met(self.cond) {
            let pc = cpu.pc();
            cpu.push_stack(pc);
            cpu.set_pc(word);
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_word();
        format!("CALL {},{:#04X}", self.cond, value)
    }
}
