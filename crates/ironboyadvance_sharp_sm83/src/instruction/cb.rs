use ironboyadvance_common::bits::BitOps;

use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::instruction::cb::bit_op::BitOp;
use crate::instruction::cb::rotate_shift_r8::RotateShiftR8;
use crate::memory::MemoryInterface;

mod bit_op;
mod rotate_shift_r8;

pub(crate) type CbInstructionFactory = fn(u8) -> CbInstruction;

#[derive(Debug, Clone, Copy)]
pub(crate) enum CbInstruction {
    RotateShiftR8(RotateShiftR8),
    BitOp(BitOp),
}

impl Instruction for CbInstruction {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        match self {
            Self::RotateShiftR8(i) => i.execute(cpu),
            Self::BitOp(i) => i.execute(cpu),
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        match self {
            Self::RotateShiftR8(i) => i.disassemble(cpu),
            Self::BitOp(i) => i.disassemble(cpu),
        }
    }
}

pub(crate) fn generate_cb_lut() -> [CbInstructionFactory; 256] {
    let mut lut: [CbInstructionFactory; 256] = [|opcode| CbInstruction::RotateShiftR8(RotateShiftR8::new(opcode)); 256];
    for (opcode, factory) in lut.iter_mut().enumerate() {
        *factory = decode_cb(opcode as u8);
    }
    lut
}

fn decode_cb(opcode: u8) -> CbInstructionFactory {
    match opcode.bits(6..=7) {
        0b00 => |opcode| CbInstruction::RotateShiftR8(RotateShiftR8::new(opcode)),
        _ => |opcode| CbInstruction::BitOp(BitOp::new(opcode)),
    }
}
