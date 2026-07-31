use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Daa;

impl Daa {
    pub(crate) fn new(_opcode: u8) -> Self {
        Self
    }
}

impl Instruction for Daa {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let mut a = cpu.registers().a();
        let carry = cpu.registers_mut().f_mut().carry();
        let half_carry = cpu.registers_mut().f_mut().half_carry();
        let subtraction = cpu.registers_mut().f_mut().subtraction();
        let mut correction = if carry { 0x60 } else { 0x00 };

        if half_carry {
            correction |= 0x06;
        }

        if !subtraction {
            if a & 0x0F > 0x09 {
                correction |= 0x06;
            }
            if a > 0x99 {
                correction |= 0x60;
            }
            a = a.wrapping_add(correction);
        } else {
            a = a.wrapping_sub(correction);
        }

        cpu.registers_mut().f_mut().set_zero(a == 0);
        cpu.registers_mut().f_mut().set_half_carry(false);
        cpu.registers_mut().f_mut().set_carry(correction >= 0x60);
        cpu.registers_mut().set_a(a);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        "DAA".to_string()
    }
}
