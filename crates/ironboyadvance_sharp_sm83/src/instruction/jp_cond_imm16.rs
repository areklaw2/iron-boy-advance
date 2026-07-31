use ironboyadvance_common::bits::BitOps;

use crate::Condition;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct JpCondImm16 {
    cond: Condition,
}

impl JpCondImm16 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            cond: Condition::from(opcode.bits(3..=4)),
        }
    }
}

impl Instruction for JpCondImm16 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let word = cpu.fetch_word();
        if cpu.is_condition_met(self.cond) {
            cpu.set_pc(word);
            cpu.bus_mut().idle_cycle();
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_word();
        format!("JP {},{:#04X}", self.cond, value)
    }
}
