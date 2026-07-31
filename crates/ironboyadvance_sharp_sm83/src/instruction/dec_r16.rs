use ironboyadvance_common::bits::BitOps;

use crate::Register16;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct DecR16 {
    r16: Register16,
}

impl DecR16 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r16: Register16::from(opcode.bits(4..=5)),
        }
    }
}

impl Instruction for DecR16 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_16(self.r16).wrapping_sub(1);
        cpu.set_register_16(self.r16, value);
        cpu.bus_mut().idle_cycle();
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("DEC {}", self.r16)
    }
}
