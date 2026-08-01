use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Stop;

impl Stop {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Stop {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        if !cpu.bus_mut().change_speed() {
            cpu.set_stopped(true);
        }
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) -> String {
        "STOP".to_string()
    }
}
