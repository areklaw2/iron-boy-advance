use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Reti;

impl Reti {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Reti {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.pop_stack();
        cpu.set_pc(value);
        cpu.bus_mut().idle_cycle();
        cpu.set_interrupt_master_enable(true);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "RETI".to_string()
    }
}
