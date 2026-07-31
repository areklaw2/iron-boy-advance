use ironboyadvance_common::bits::BitOps;

use crate::Register16;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct AddHlR16 {
    r16: Register16,
}

impl AddHlR16 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            r16: Register16::from(opcode.bits(4..=5)),
        }
    }
}

impl Instruction for AddHlR16 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value1 = cpu.registers().hl();
        let value2 = cpu.register_16(self.r16);
        let result = value1.wrapping_add(value2);
        cpu.bus_mut().idle_cycle();

        cpu.registers_mut().set_hl(result);
        cpu.registers_mut().f_mut().set_subtraction(false);
        cpu.registers_mut()
            .f_mut()
            .set_half_carry((value1 & 0x0FFF) + (value2 & 0x0FFF) > 0x0FFF);
        cpu.registers_mut().f_mut().set_carry(value1 as u32 + value2 as u32 > 0xFFFF);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("ADD HL,{}", self.r16)
    }
}
