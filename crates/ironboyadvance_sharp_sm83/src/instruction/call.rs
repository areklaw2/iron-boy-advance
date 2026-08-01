use ironboyadvance_common::bits::BitOps;

use crate::Condition;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

const CALL_IMM16: u8 = 0xCD;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Call {
    condition: Option<Condition>,
}

impl Call {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            condition: match opcode {
                CALL_IMM16 => None,
                _ => Some(Condition::from(opcode.bits(3..=4))),
            },
        }
    }
}

impl Instruction for Call {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let address = cpu.fetch_word();
        let taken = match self.condition {
            Some(condition) => cpu.is_condition_met(condition),
            None => true,
        };

        if taken {
            let return_address = cpu.pc();
            cpu.push_stack(return_address);
            cpu.set_pc(address);
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_word();
        match self.condition {
            Some(condition) => format!("CALL {},{:#04X}", condition, value),
            None => format!("CALL {:#04X}", value),
        }
    }
}
