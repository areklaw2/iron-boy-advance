use ironboyadvance_common::bits::BitOps;

use crate::Condition;
use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RelativeJump {
    condition: Option<Condition>,
}

impl RelativeJump {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            condition: match opcode.bit(5) {
                true => Some(Condition::from(opcode.bits(3..=4))),
                false => None,
            },
        }
    }
}

impl Instruction for RelativeJump {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        let offset = cpu.fetch_byte() as i8;

        let taken = match self.condition {
            Some(condition) => cpu.is_condition_met(condition),
            None => true,
        };

        if taken {
            let pc = ((cpu.pc() as i32) + (offset as i32)) as u16;
            cpu.set_pc(pc);
            cpu.bus_mut().idle_cycle();
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) -> String {
        let value = cpu.fetch_byte();
        match self.condition {
            Some(condition) => format!("JR {},{:#04X}", condition, value),
            None => format!("JR {:#04X}", value),
        }
    }
}
