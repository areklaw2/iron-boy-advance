use ironboyadvance_common::bits::BitOps;

use crate::AccumulatorOpcode;
use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) struct AccumulatorOperations {
    opcode: AccumulatorOpcode,
}

impl AccumulatorOperations {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            opcode: AccumulatorOpcode::from(opcode.bits(3..=5)),
        }
    }
}

impl Instruction for AccumulatorOperations {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        match self.opcode {
            AccumulatorOpcode::RLCA => {
                let a = cpu.registers().a();
                let carry = a & 0x80 == 0x80;
                let result = (a << 1) | (if carry { 1 } else { 0 });

                set_rotate_flags(cpu, carry);
                cpu.registers_mut().set_a(result);
            }
            AccumulatorOpcode::RRCA => {
                let a = cpu.registers().a();
                let carry = a & 0x01 == 0x01;
                let result = (a >> 1) | (if carry { 0x80 } else { 0 });

                set_rotate_flags(cpu, carry);
                cpu.registers_mut().set_a(result);
            }
            AccumulatorOpcode::RLA => {
                let a = cpu.registers().a();
                let old_carry = cpu.registers().f().carry();
                let carry = a & 0x80 == 0x80;
                let result = (a << 1) | (if old_carry { 1 } else { 0 });

                set_rotate_flags(cpu, carry);
                cpu.registers_mut().set_a(result);
            }
            AccumulatorOpcode::RRA => {
                let a = cpu.registers().a();
                let old_carry = cpu.registers().f().carry();
                let carry = a & 0x01 == 0x01;
                let result = (a >> 1) | (if old_carry { 0x80 } else { 0 });

                set_rotate_flags(cpu, carry);
                cpu.registers_mut().set_a(result);
            }
            AccumulatorOpcode::DAA => {
                let mut a = cpu.registers().a();
                let carry = cpu.registers().f().carry();
                let half_carry = cpu.registers().f().half_carry();
                let subtraction = cpu.registers().f().subtraction();
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
            AccumulatorOpcode::CPL => {
                let not_a = !cpu.registers().a();
                cpu.registers_mut().set_a(not_a);
                cpu.registers_mut().f_mut().set_subtraction(true);
                cpu.registers_mut().f_mut().set_half_carry(true);
            }
            AccumulatorOpcode::SCF => {
                cpu.registers_mut().f_mut().set_carry(true);
                cpu.registers_mut().f_mut().set_half_carry(false);
                cpu.registers_mut().f_mut().set_subtraction(false);
            }
            AccumulatorOpcode::CCF => {
                let carry = !cpu.registers().f().carry();
                cpu.registers_mut().f_mut().set_carry(carry);
                cpu.registers_mut().f_mut().set_half_carry(false);
                cpu.registers_mut().f_mut().set_subtraction(false);
            }
        }
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) -> String {
        self.opcode.to_string()
    }
}

fn set_rotate_flags<I: MemoryInterface>(cpu: &mut Sm83<I>, carry: bool) {
    cpu.registers_mut().f_mut().set_zero(false);
    cpu.registers_mut().f_mut().set_subtraction(false);
    cpu.registers_mut().f_mut().set_half_carry(false);
    cpu.registers_mut().f_mut().set_carry(carry);
}
