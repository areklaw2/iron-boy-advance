use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::instruction::cb::generate_cb_lut;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Prefix;

impl Prefix {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Prefix {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        let opcode = cpu.fetch_byte();
        let cb_lut = generate_cb_lut();
        let instruction = (cb_lut[opcode as usize])(opcode);
        instruction.execute(cpu);
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) -> String {
        let opcode = cpu.fetch_byte();
        let cb_lut = generate_cb_lut();
        let instruction = (cb_lut[opcode as usize])(opcode);
        instruction.disassemble(cpu)
    }
}
