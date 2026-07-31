use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct JrImm8;

impl JrImm8 {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for JrImm8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let signed = cpu.fetch_byte() as i8;
        let pc = ((cpu.pc() as u32 as i32) + (signed as i32)) as u16;
        cpu.set_pc(pc);
        cpu.bus_mut().idle_cycle();
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let value = cpu.fetch_byte();
        format!("JR {:#04X}", value)
    }
}
