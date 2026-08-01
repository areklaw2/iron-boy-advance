use ironboyadvance_common::bits::BitOps;

use crate::Register16;
use crate::alu;
use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

const ADD_SP_SIGNED_IMM8: u8 = 0xE8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Alu16Opcode {
    Increment(Register16),
    Decrement(Register16),
    AddToHl(Register16),
    AddToStackPointer,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Alu16 {
    opcode: Alu16Opcode,
}

impl Alu16 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            opcode: match opcode {
                ADD_SP_SIGNED_IMM8 => Alu16Opcode::AddToStackPointer,
                _ => {
                    let r16 = Register16::from(opcode.bits(4..=5));
                    match opcode.bits(0..=3) {
                        0b0011 => Alu16Opcode::Increment(r16),
                        0b1011 => Alu16Opcode::Decrement(r16),
                        _ => Alu16Opcode::AddToHl(r16),
                    }
                }
            },
        }
    }
}

impl Instruction for Alu16 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        match self.opcode {
            Alu16Opcode::Increment(r16) => {
                let value = cpu.register_16(r16).wrapping_add(1);
                cpu.set_register_16(r16, value);
                cpu.bus_mut().idle_cycle();
            }
            Alu16Opcode::Decrement(r16) => {
                let value = cpu.register_16(r16).wrapping_sub(1);
                cpu.set_register_16(r16, value);
                cpu.bus_mut().idle_cycle();
            }
            Alu16Opcode::AddToHl(r16) => {
                let hl = cpu.registers().hl();
                let operand = cpu.register_16(r16);
                let result = hl.wrapping_add(operand);
                cpu.bus_mut().idle_cycle();

                cpu.registers_mut().set_hl(result);
                cpu.registers_mut().f_mut().set_subtraction(false);
                cpu.registers_mut()
                    .f_mut()
                    .set_half_carry((hl & 0x0FFF) + (operand & 0x0FFF) > 0x0FFF);
                cpu.registers_mut().f_mut().set_carry(hl as u32 + operand as u32 > 0xFFFF);
            }
            Alu16Opcode::AddToStackPointer => {
                let offset = cpu.fetch_byte() as i8 as i16 as u16;
                cpu.bus_mut().idle_cycle();
                let result = alu::add_offset_to_stack_pointer(cpu, offset);
                cpu.registers_mut().set_sp(result);
                cpu.bus_mut().idle_cycle();
            }
        }
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Sm83<I>) -> String {
        match self.opcode {
            Alu16Opcode::Increment(r16) => format!("INC {}", r16),
            Alu16Opcode::Decrement(r16) => format!("DEC {}", r16),
            Alu16Opcode::AddToHl(r16) => format!("ADD HL,{}", r16),
            Alu16Opcode::AddToStackPointer => "ADD SP,u8".to_string(),
        }
    }
}
