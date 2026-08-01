use ironboyadvance_common::bits::BitOps;

use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;
use crate::{Register8, RotateShiftOpcode};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RotateShiftR8 {
    opcode: RotateShiftOpcode,
    r8: Register8,
}

impl RotateShiftR8 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            opcode: RotateShiftOpcode::from(opcode.bits(3..=5)),
            r8: Register8::from(opcode.bits(0..=2)),
        }
    }
}

impl Instruction for RotateShiftR8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let value = cpu.register_8(self.r8);
        let (result, carry) = match self.opcode {
            RotateShiftOpcode::RLC => {
                let carry = value & 0x80 == 0x80;
                ((value << 1) | (if carry { 1 } else { 0 }), carry)
            }
            RotateShiftOpcode::RRC => {
                let carry = value & 0x01 == 0x01;
                ((value >> 1) | (if carry { 0x80 } else { 0 }), carry)
            }
            RotateShiftOpcode::RL => {
                let old_carry = cpu.registers().f().carry();
                ((value << 1) | (if old_carry { 1 } else { 0 }), value & 0x80 == 0x80)
            }
            RotateShiftOpcode::RR => {
                let old_carry = cpu.registers().f().carry();
                ((value >> 1) | (if old_carry { 0x80 } else { 0 }), value & 0x01 == 0x01)
            }
            RotateShiftOpcode::SLA => (value << 1, value & 0x80 == 0x80),
            RotateShiftOpcode::SRA => ((value >> 1) | (value & 0x80), value & 0x01 == 0x01),
            RotateShiftOpcode::SWAP => (value.rotate_left(4), false),
            RotateShiftOpcode::SRL => (value >> 1, value & 0x01 == 0x01),
        };

        cpu.set_register_8(self.r8, result);

        cpu.registers_mut().f_mut().set_zero(result == 0);
        cpu.registers_mut().f_mut().set_subtraction(false);
        cpu.registers_mut().f_mut().set_half_carry(false);
        cpu.registers_mut().f_mut().set_carry(carry);
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut SharpSm83<I>) -> String {
        format!("{} {}", self.opcode, self.r8)
    }
}
