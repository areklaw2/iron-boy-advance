use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct AddSpSignedImm8;

impl AddSpSignedImm8 {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for AddSpSignedImm8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value1 = cpu.registers().sp();
        let value2 = cpu.fetch_byte() as i8 as i16 as u16;
        cpu.bus_mut().idle_cycle();
        let result = value1.wrapping_add(value2);
        cpu.registers_mut().set_sp(result);
        cpu.bus_mut().idle_cycle();

        cpu.registers_mut().f_mut().set_zero(false);
        cpu.registers_mut().f_mut().set_subtraction(false);
        cpu.registers_mut()
            .f_mut()
            .set_half_carry((value1 & 0x000F) + (value2 & 0x000F) > 0x000F);
        cpu.registers_mut()
            .f_mut()
            .set_carry((value1 & 0x00FF) + (value2 & 0x00FF) > 0x00FF);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "ADD SP,u8".to_string()
    }
}
