use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::instruction::cb::bit::Bit;
use crate::instruction::cb::res::Res;
use crate::instruction::cb::rl::Rl;
use crate::instruction::cb::rlc::Rlc;
use crate::instruction::cb::rr::Rr;
use crate::instruction::cb::rrc::Rrc;
use crate::instruction::cb::set::Set;
use crate::instruction::cb::sla::Sla;
use crate::instruction::cb::sra::Sra;
use crate::instruction::cb::srl::Srl;
use crate::instruction::cb::swap::Swap;
use crate::memory::MemoryInterface;
use ironboyadvance_common::bits::BitOps;

mod bit;
mod res;
mod rl;
mod rlc;
mod rr;
mod rrc;
mod set;
mod sla;
mod sra;
mod srl;
mod swap;

pub(crate) type CbInstructionFactory = fn(u8) -> CbInstruction;

#[derive(Debug, Clone, Copy)]
pub(crate) enum CbInstruction {
    Rlc(Rlc),
    Rrc(Rrc),
    Rl(Rl),
    Rr(Rr),
    Sla(Sla),
    Sra(Sra),
    Swap(Swap),
    Srl(Srl),
    Bit(Bit),
    Res(Res),
    Set(Set),
}

impl Instruction for CbInstruction {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        match self {
            Self::Rlc(i) => i.execute(cpu),
            Self::Rrc(i) => i.execute(cpu),
            Self::Rl(i) => i.execute(cpu),
            Self::Rr(i) => i.execute(cpu),
            Self::Sla(i) => i.execute(cpu),
            Self::Sra(i) => i.execute(cpu),
            Self::Swap(i) => i.execute(cpu),
            Self::Srl(i) => i.execute(cpu),
            Self::Bit(i) => i.execute(cpu),
            Self::Res(i) => i.execute(cpu),
            Self::Set(i) => i.execute(cpu),
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        match self {
            Self::Rlc(i) => i.disassemble(cpu),
            Self::Rrc(i) => i.disassemble(cpu),
            Self::Rl(i) => i.disassemble(cpu),
            Self::Rr(i) => i.disassemble(cpu),
            Self::Sla(i) => i.disassemble(cpu),
            Self::Sra(i) => i.disassemble(cpu),
            Self::Swap(i) => i.disassemble(cpu),
            Self::Srl(i) => i.disassemble(cpu),
            Self::Bit(i) => i.disassemble(cpu),
            Self::Res(i) => i.disassemble(cpu),
            Self::Set(i) => i.disassemble(cpu),
        }
    }
}

pub(crate) fn generate_cb_lut() -> [CbInstructionFactory; 256] {
    let mut lut: [CbInstructionFactory; 256] = [|opcode| CbInstruction::Rlc(Rlc::new(opcode)); 256];
    for (opcode, factory) in lut.iter_mut().enumerate() {
        *factory = decode_cb(opcode as u8);
    }
    lut
}

fn decode_cb(opcode: u8) -> CbInstructionFactory {
    match opcode.bits(6..=7) {
        0b01 => |opcode| CbInstruction::Bit(Bit::new(opcode)),
        0b10 => |opcode| CbInstruction::Res(Res::new(opcode)),
        0b11 => |opcode| CbInstruction::Set(Set::new(opcode)),
        _ => match opcode.bits(3..=5) {
            0b000 => |opcode| CbInstruction::Rlc(Rlc::new(opcode)),
            0b001 => |opcode| CbInstruction::Rrc(Rrc::new(opcode)),
            0b010 => |opcode| CbInstruction::Rl(Rl::new(opcode)),
            0b011 => |opcode| CbInstruction::Rr(Rr::new(opcode)),
            0b100 => |opcode| CbInstruction::Sla(Sla::new(opcode)),
            0b101 => |opcode| CbInstruction::Sra(Sra::new(opcode)),
            0b110 => |opcode| CbInstruction::Swap(Swap::new(opcode)),
            _ => |opcode| CbInstruction::Srl(Srl::new(opcode)),
        },
    }
}

pub(crate) fn set_rotate_shift_flags<I: MemoryInterface>(cpu: &mut SharpSm83<I>, result: u8, carry: bool) {
    cpu.registers_mut().f_mut().set_zero(result == 0);
    cpu.registers_mut().f_mut().set_subtraction(false);
    cpu.registers_mut().f_mut().set_half_carry(false);
    cpu.registers_mut().f_mut().set_carry(carry);
}
