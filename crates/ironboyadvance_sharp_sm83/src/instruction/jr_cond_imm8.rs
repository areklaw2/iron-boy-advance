use ironboyadvance_common::bits::BitOps;

use crate::Condition;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct JrCondImm8 {
    cond: Condition,
}

impl JrCondImm8 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            cond: Condition::from(opcode.bits(3..=4)),
        }
    }
}

impl Instruction for JrCondImm8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        if cpu.is_condition_met(self.cond) {
            let signed = cpu.fetch_byte() as i8;
            let pc = ((cpu.pc() as i16) + (signed as i16)) as u16;
            cpu.set_pc(pc);
        } else {
            cpu.set_pc(cpu.pc() + 1);
        }
        cpu.bus_mut().idle_cycle();
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_byte();
        format!("JR {},{:#04X}", self.cond, value)
    }
}
