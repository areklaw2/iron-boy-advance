use ironboyadvance_common::bits::BitOps;

use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RstTgt3 {
    target: u16,
}

impl RstTgt3 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            target: opcode.bits(3..=5) as u16 * 8,
        }
    }
}

impl Instruction for RstTgt3 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let pc = cpu.pc();
        cpu.push_stack(pc);
        cpu.set_pc(self.target);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("RST {:02X}H", self.target)
    }
}
