use ironboyadvance_common::bits::BitOps;

use crate::Condition;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

const RET: u8 = 0xC9;
const RETI: u8 = 0xD9;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Ret {
    condition: Option<Condition>,
    enables_interrupts: bool,
}

impl Ret {
    pub(crate) fn new(opcode: u8) -> Self {
        match opcode {
            RET => Self {
                condition: None,
                enables_interrupts: false,
            },
            RETI => Self {
                condition: None,
                enables_interrupts: true,
            },
            _ => Self {
                condition: Some(Condition::from(opcode.bits(3..=4))),
                enables_interrupts: false,
            },
        }
    }
}

impl Instruction for Ret {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        match self.condition {
            Some(condition) => {
                if cpu.is_condition_met(condition) {
                    return_to_caller(cpu);
                }
                cpu.bus_mut().idle_cycle();
            }
            None => {
                return_to_caller(cpu);
                if self.enables_interrupts {
                    cpu.set_interrupt_master_enable(true);
                }
            }
        }
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        match (self.condition, self.enables_interrupts) {
            (Some(condition), _) => format!("RET {}", condition),
            (None, true) => "RETI".to_string(),
            (None, false) => "RET".to_string(),
        }
    }
}

fn return_to_caller<I: MemoryInterface>(cpu: &mut SharpSm83<I>) {
    let address = cpu.pop_stack();
    cpu.set_pc(address);
    cpu.bus_mut().idle_cycle();
}
