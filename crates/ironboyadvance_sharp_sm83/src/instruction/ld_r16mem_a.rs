use ironboyadvance_common::bits::BitOps;

use crate::R16Memory;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LdR16MemA {
    r16mem: R16Memory,
}

impl LdR16MemA {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r16mem: R16Memory::from(opcode.bits(4..=5)),
        }
    }
}

impl Instruction for LdR16MemA {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let address = cpu.register_16_memory(self.r16mem);
        let value = cpu.registers().a();
        cpu.bus_mut().store_8(address, value);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("LD [{}],A", self.r16mem)
    }
}
