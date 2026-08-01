use ironboyadvance_common::bits::BitOps;

use crate::Condition;
use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

const JP_IMM16: u8 = 0xC3;
const JP_HL: u8 = 0xE9;

#[derive(Debug, Clone, Copy)]
pub(crate) enum JumpTarget {
    Immediate(Option<Condition>),
    Hl,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Jump {
    target: JumpTarget,
}

impl Jump {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            target: match opcode {
                JP_IMM16 => JumpTarget::Immediate(None),
                JP_HL => JumpTarget::Hl,
                _ => JumpTarget::Immediate(Some(Condition::from(opcode.bits(3..=4)))),
            },
        }
    }
}

impl Instruction for Jump {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        match self.target {
            JumpTarget::Hl => {
                let hl = cpu.registers().hl();
                cpu.set_pc(hl);
            }
            JumpTarget::Immediate(condition) => {
                let address = cpu.fetch_word();
                let taken = match condition {
                    Some(condition) => cpu.is_condition_met(condition),
                    None => true,
                };

                if taken {
                    cpu.set_pc(address);
                    cpu.bus_mut().idle_cycle();
                }
            }
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) -> String {
        match self.target {
            JumpTarget::Hl => "JP HL".to_string(),
            JumpTarget::Immediate(condition) => {
                let value = cpu.fetch_word();
                match condition {
                    Some(condition) => format!("JP {},{:#04X}", condition, value),
                    None => format!("JP {:#04X}", value),
                }
            }
        }
    }
}
