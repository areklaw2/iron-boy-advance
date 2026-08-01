use ironboyadvance_common::bits::BitOps;

use crate::alu;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;
use crate::{AluOpcode, Register8};

#[derive(Debug, Clone, Copy)]
pub(crate) enum AluOperand {
    Register(Register8),
    Immediate,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Alu8 {
    opcode: AluOpcode,
    operand: AluOperand,
}

impl Alu8 {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            opcode: AluOpcode::from(opcode.bits(3..=5)),
            operand: match opcode.bit(6) {
                true => AluOperand::Immediate,
                false => AluOperand::Register(Register8::from(opcode.bits(0..=2))),
            },
        }
    }
}

impl Instruction for Alu8 {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let operand = match self.operand {
            AluOperand::Register(r8) => cpu.register_8(r8),
            AluOperand::Immediate => cpu.fetch_byte(),
        };

        match self.opcode {
            AluOpcode::ADD => alu::add(cpu, operand),
            AluOpcode::ADC => alu::adc(cpu, operand),
            AluOpcode::SUB => alu::sub(cpu, operand),
            AluOpcode::SBC => alu::sbc(cpu, operand),
            AluOpcode::AND => alu::and(cpu, operand),
            AluOpcode::XOR => alu::xor(cpu, operand),
            AluOpcode::OR => alu::or(cpu, operand),
            AluOpcode::CP => alu::cp(cpu, operand),
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        match self.operand {
            AluOperand::Register(r8) => format!("{} A,{}", self.opcode, r8),
            AluOperand::Immediate => format!("{} A,{:#04X}", self.opcode, cpu.fetch_byte()),
        }
    }
}
