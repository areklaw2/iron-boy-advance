use crate::cpu::SharpSm83;
use crate::instruction::adc_a_imm8::AdcAImm8;
use crate::instruction::adc_a_r8::AdcAR8;
use crate::instruction::add_a_imm8::AddAImm8;
use crate::instruction::add_a_r8::AddAR8;
use crate::instruction::add_hl_r16::AddHlR16;
use crate::instruction::add_sp_signed_imm8::AddSpSignedImm8;
use crate::instruction::and_a_imm8::AndAImm8;
use crate::instruction::and_a_r8::AndAR8;
use crate::instruction::call_cond_imm16::CallCondImm16;
use crate::instruction::call_imm16::CallImm16;
use crate::instruction::ccf::Ccf;
use crate::instruction::cp_a_imm8::CpAImm8;
use crate::instruction::cp_a_r8::CpAR8;
use crate::instruction::cpl::Cpl;
use crate::instruction::daa::Daa;
use crate::instruction::dec_r8::DecR8;
use crate::instruction::dec_r16::DecR16;
use crate::instruction::di::Di;
use crate::instruction::ei::Ei;
use crate::instruction::halt::Halt;
use crate::instruction::inc_r8::IncR8;
use crate::instruction::inc_r16::IncR16;
use crate::instruction::jp_cond_imm16::JpCondImm16;
use crate::instruction::jp_hl::JpHl;
use crate::instruction::jp_imm16::JpImm16;
use crate::instruction::jr_cond_imm8::JrCondImm8;
use crate::instruction::jr_imm8::JrImm8;
use crate::instruction::ld_a_imm16mem::LdAImm16Mem;
use crate::instruction::ld_a_r16mem::LdAR16Mem;
use crate::instruction::ld_hl_sp_plus_signed_imm8::LdHlSpPlusSignedImm8;
use crate::instruction::ld_imm16_sp::LdImm16Sp;
use crate::instruction::ld_imm16mem_a::LdImm16MemA;
use crate::instruction::ld_r8_imm8::LdR8Imm8;
use crate::instruction::ld_r8_r8::LdR8R8;
use crate::instruction::ld_r16_imm16::LdR16Imm16;
use crate::instruction::ld_r16mem_a::LdR16MemA;
use crate::instruction::ld_sp_hl::LdSpHl;
use crate::instruction::ldh_a_cmem::LdhACMem;
use crate::instruction::ldh_a_imm8mem::LdhAImm8Mem;
use crate::instruction::ldh_cmem_a::LdhCMemA;
use crate::instruction::ldh_imm8mem_a::LdhImm8MemA;
use crate::instruction::nop::Nop;
use crate::instruction::or_a_imm8::OrAImm8;
use crate::instruction::or_a_r8::OrAR8;
use crate::instruction::pop_r16stk::PopR16Stk;
use crate::instruction::prefix::Prefix;
use crate::instruction::push_r16stk::PushR16Stk;
use crate::instruction::ret::Ret;
use crate::instruction::ret_cond::RetCond;
use crate::instruction::reti::Reti;
use crate::instruction::rla::Rla;
use crate::instruction::rlca::Rlca;
use crate::instruction::rra::Rra;
use crate::instruction::rrca::Rrca;
use crate::instruction::rst_tgt3::RstTgt3;
use crate::instruction::sbc_a_imm8::SbcAImm8;
use crate::instruction::sbc_a_r8::SbcAR8;
use crate::instruction::scf::Scf;
use crate::instruction::stop::Stop;
use crate::instruction::sub_a_imm8::SubAImm8;
use crate::instruction::sub_a_r8::SubAR8;
use crate::instruction::undefined::Undefined;
use crate::instruction::xor_a_imm8::XorAImm8;
use crate::instruction::xor_a_r8::XorAR8;
use crate::memory::MemoryInterface;

mod adc_a_imm8;
mod adc_a_r8;
mod add_a_imm8;
mod add_a_r8;
mod add_hl_r16;
mod add_sp_signed_imm8;
mod and_a_imm8;
mod and_a_r8;
mod call_cond_imm16;
mod call_imm16;
mod cb;
mod ccf;
mod cp_a_imm8;
mod cp_a_r8;
mod cpl;
mod daa;
mod dec_r16;
mod dec_r8;
mod di;
mod ei;
mod halt;
mod inc_r16;
mod inc_r8;
mod jp_cond_imm16;
mod jp_hl;
mod jp_imm16;
mod jr_cond_imm8;
mod jr_imm8;
mod ld_a_imm16mem;
mod ld_a_r16mem;
mod ld_hl_sp_plus_signed_imm8;
mod ld_imm16_sp;
mod ld_imm16mem_a;
mod ld_r16_imm16;
mod ld_r16mem_a;
mod ld_r8_imm8;
mod ld_r8_r8;
mod ld_sp_hl;
mod ldh_a_cmem;
mod ldh_a_imm8mem;
mod ldh_cmem_a;
mod ldh_imm8mem_a;
mod nop;
mod or_a_imm8;
mod or_a_r8;
mod pop_r16stk;
mod prefix;
mod push_r16stk;
mod ret;
mod ret_cond;
mod reti;
mod rla;
mod rlca;
mod rra;
mod rrca;
mod rst_tgt3;
mod sbc_a_imm8;
mod sbc_a_r8;
mod scf;
mod stop;
mod sub_a_imm8;
mod sub_a_r8;
mod undefined;
mod xor_a_imm8;
mod xor_a_r8;

pub(crate) trait Instruction {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>);
    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String;
}

pub(crate) type SharpSm83InstructionFactory = fn(u8) -> SharpSm83Instruction;

#[derive(Debug, Clone, Copy)]
pub(crate) enum SharpSm83Instruction {
    Nop(Nop),
    Undefined(Undefined),
    LdR8Imm8(LdR8Imm8),
    LdR8R8(LdR8R8),
    Halt(Halt),
    LdR16Imm16(LdR16Imm16),
    LdImm16Sp(LdImm16Sp),
    LdR16MemA(LdR16MemA),
    LdAR16Mem(LdAR16Mem),
    IncR16(IncR16),
    DecR16(DecR16),
    AddHlR16(AddHlR16),
    IncR8(IncR8),
    DecR8(DecR8),
    Rlca(Rlca),
    Rrca(Rrca),
    Rla(Rla),
    Rra(Rra),
    Daa(Daa),
    Cpl(Cpl),
    Scf(Scf),
    Ccf(Ccf),
    JrImm8(JrImm8),
    JrCondImm8(JrCondImm8),
    Stop(Stop),
    AddAR8(AddAR8),
    AdcAR8(AdcAR8),
    SubAR8(SubAR8),
    SbcAR8(SbcAR8),
    AndAR8(AndAR8),
    XorAR8(XorAR8),
    OrAR8(OrAR8),
    CpAR8(CpAR8),
    AddAImm8(AddAImm8),
    AdcAImm8(AdcAImm8),
    SubAImm8(SubAImm8),
    SbcAImm8(SbcAImm8),
    AndAImm8(AndAImm8),
    XorAImm8(XorAImm8),
    OrAImm8(OrAImm8),
    CpAImm8(CpAImm8),
    RetCond(RetCond),
    Ret(Ret),
    Reti(Reti),
    JpCondImm16(JpCondImm16),
    JpImm16(JpImm16),
    JpHl(JpHl),
    CallCondImm16(CallCondImm16),
    CallImm16(CallImm16),
    RstTgt3(RstTgt3),
    PopR16Stk(PopR16Stk),
    PushR16Stk(PushR16Stk),
    LdhCMemA(LdhCMemA),
    LdhImm8MemA(LdhImm8MemA),
    LdImm16MemA(LdImm16MemA),
    LdhACMem(LdhACMem),
    LdhAImm8Mem(LdhAImm8Mem),
    LdAImm16Mem(LdAImm16Mem),
    AddSpSignedImm8(AddSpSignedImm8),
    LdHlSpPlusSignedImm8(LdHlSpPlusSignedImm8),
    LdSpHl(LdSpHl),
    Di(Di),
    Ei(Ei),
    Prefix(Prefix),
}

impl Instruction for SharpSm83Instruction {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        match self {
            Self::Nop(i) => i.execute(cpu),
            Self::Undefined(i) => i.execute(cpu),
            Self::LdR8Imm8(i) => i.execute(cpu),
            Self::LdR8R8(i) => i.execute(cpu),
            Self::Halt(i) => i.execute(cpu),
            Self::LdR16Imm16(i) => i.execute(cpu),
            Self::LdImm16Sp(i) => i.execute(cpu),
            Self::LdR16MemA(i) => i.execute(cpu),
            Self::LdAR16Mem(i) => i.execute(cpu),
            Self::IncR16(i) => i.execute(cpu),
            Self::DecR16(i) => i.execute(cpu),
            Self::AddHlR16(i) => i.execute(cpu),
            Self::IncR8(i) => i.execute(cpu),
            Self::DecR8(i) => i.execute(cpu),
            Self::Rlca(i) => i.execute(cpu),
            Self::Rrca(i) => i.execute(cpu),
            Self::Rla(i) => i.execute(cpu),
            Self::Rra(i) => i.execute(cpu),
            Self::Daa(i) => i.execute(cpu),
            Self::Cpl(i) => i.execute(cpu),
            Self::Scf(i) => i.execute(cpu),
            Self::Ccf(i) => i.execute(cpu),
            Self::JrImm8(i) => i.execute(cpu),
            Self::JrCondImm8(i) => i.execute(cpu),
            Self::Stop(i) => i.execute(cpu),
            Self::AddAR8(i) => i.execute(cpu),
            Self::AdcAR8(i) => i.execute(cpu),
            Self::SubAR8(i) => i.execute(cpu),
            Self::SbcAR8(i) => i.execute(cpu),
            Self::AndAR8(i) => i.execute(cpu),
            Self::XorAR8(i) => i.execute(cpu),
            Self::OrAR8(i) => i.execute(cpu),
            Self::CpAR8(i) => i.execute(cpu),
            Self::AddAImm8(i) => i.execute(cpu),
            Self::AdcAImm8(i) => i.execute(cpu),
            Self::SubAImm8(i) => i.execute(cpu),
            Self::SbcAImm8(i) => i.execute(cpu),
            Self::AndAImm8(i) => i.execute(cpu),
            Self::XorAImm8(i) => i.execute(cpu),
            Self::OrAImm8(i) => i.execute(cpu),
            Self::CpAImm8(i) => i.execute(cpu),
            Self::RetCond(i) => i.execute(cpu),
            Self::Ret(i) => i.execute(cpu),
            Self::Reti(i) => i.execute(cpu),
            Self::JpCondImm16(i) => i.execute(cpu),
            Self::JpImm16(i) => i.execute(cpu),
            Self::JpHl(i) => i.execute(cpu),
            Self::CallCondImm16(i) => i.execute(cpu),
            Self::CallImm16(i) => i.execute(cpu),
            Self::RstTgt3(i) => i.execute(cpu),
            Self::PopR16Stk(i) => i.execute(cpu),
            Self::PushR16Stk(i) => i.execute(cpu),
            Self::LdhCMemA(i) => i.execute(cpu),
            Self::LdhImm8MemA(i) => i.execute(cpu),
            Self::LdImm16MemA(i) => i.execute(cpu),
            Self::LdhACMem(i) => i.execute(cpu),
            Self::LdhAImm8Mem(i) => i.execute(cpu),
            Self::LdAImm16Mem(i) => i.execute(cpu),
            Self::AddSpSignedImm8(i) => i.execute(cpu),
            Self::LdHlSpPlusSignedImm8(i) => i.execute(cpu),
            Self::LdSpHl(i) => i.execute(cpu),
            Self::Di(i) => i.execute(cpu),
            Self::Ei(i) => i.execute(cpu),
            Self::Prefix(i) => i.execute(cpu),
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        match self {
            Self::Nop(i) => i.disassemble(cpu),
            Self::Undefined(i) => i.disassemble(cpu),
            Self::LdR8Imm8(i) => i.disassemble(cpu),
            Self::LdR8R8(i) => i.disassemble(cpu),
            Self::Halt(i) => i.disassemble(cpu),
            Self::LdR16Imm16(i) => i.disassemble(cpu),
            Self::LdImm16Sp(i) => i.disassemble(cpu),
            Self::LdR16MemA(i) => i.disassemble(cpu),
            Self::LdAR16Mem(i) => i.disassemble(cpu),
            Self::IncR16(i) => i.disassemble(cpu),
            Self::DecR16(i) => i.disassemble(cpu),
            Self::AddHlR16(i) => i.disassemble(cpu),
            Self::IncR8(i) => i.disassemble(cpu),
            Self::DecR8(i) => i.disassemble(cpu),
            Self::Rlca(i) => i.disassemble(cpu),
            Self::Rrca(i) => i.disassemble(cpu),
            Self::Rla(i) => i.disassemble(cpu),
            Self::Rra(i) => i.disassemble(cpu),
            Self::Daa(i) => i.disassemble(cpu),
            Self::Cpl(i) => i.disassemble(cpu),
            Self::Scf(i) => i.disassemble(cpu),
            Self::Ccf(i) => i.disassemble(cpu),
            Self::JrImm8(i) => i.disassemble(cpu),
            Self::JrCondImm8(i) => i.disassemble(cpu),
            Self::Stop(i) => i.disassemble(cpu),
            Self::AddAR8(i) => i.disassemble(cpu),
            Self::AdcAR8(i) => i.disassemble(cpu),
            Self::SubAR8(i) => i.disassemble(cpu),
            Self::SbcAR8(i) => i.disassemble(cpu),
            Self::AndAR8(i) => i.disassemble(cpu),
            Self::XorAR8(i) => i.disassemble(cpu),
            Self::OrAR8(i) => i.disassemble(cpu),
            Self::CpAR8(i) => i.disassemble(cpu),
            Self::AddAImm8(i) => i.disassemble(cpu),
            Self::AdcAImm8(i) => i.disassemble(cpu),
            Self::SubAImm8(i) => i.disassemble(cpu),
            Self::SbcAImm8(i) => i.disassemble(cpu),
            Self::AndAImm8(i) => i.disassemble(cpu),
            Self::XorAImm8(i) => i.disassemble(cpu),
            Self::OrAImm8(i) => i.disassemble(cpu),
            Self::CpAImm8(i) => i.disassemble(cpu),
            Self::RetCond(i) => i.disassemble(cpu),
            Self::Ret(i) => i.disassemble(cpu),
            Self::Reti(i) => i.disassemble(cpu),
            Self::JpCondImm16(i) => i.disassemble(cpu),
            Self::JpImm16(i) => i.disassemble(cpu),
            Self::JpHl(i) => i.disassemble(cpu),
            Self::CallCondImm16(i) => i.disassemble(cpu),
            Self::CallImm16(i) => i.disassemble(cpu),
            Self::RstTgt3(i) => i.disassemble(cpu),
            Self::PopR16Stk(i) => i.disassemble(cpu),
            Self::PushR16Stk(i) => i.disassemble(cpu),
            Self::LdhCMemA(i) => i.disassemble(cpu),
            Self::LdhImm8MemA(i) => i.disassemble(cpu),
            Self::LdImm16MemA(i) => i.disassemble(cpu),
            Self::LdhACMem(i) => i.disassemble(cpu),
            Self::LdhAImm8Mem(i) => i.disassemble(cpu),
            Self::LdAImm16Mem(i) => i.disassemble(cpu),
            Self::AddSpSignedImm8(i) => i.disassemble(cpu),
            Self::LdHlSpPlusSignedImm8(i) => i.disassemble(cpu),
            Self::LdSpHl(i) => i.disassemble(cpu),
            Self::Di(i) => i.disassemble(cpu),
            Self::Ei(i) => i.disassemble(cpu),
            Self::Prefix(i) => i.disassemble(cpu),
        }
    }
}

pub(crate) fn generate_lut() -> [SharpSm83InstructionFactory; 256] {
    let mut lut: [SharpSm83InstructionFactory; 256] =
        [|opcode| SharpSm83Instruction::Undefined(Undefined::new(opcode)); 256];
    for (opcode, factory) in lut.iter_mut().enumerate() {
        *factory = decode(opcode as u8);
    }
    lut
}

fn decode(opcode: u8) -> SharpSm83InstructionFactory {
    match opcode {
        0x00 => |opcode| SharpSm83Instruction::Nop(Nop::new(opcode)),
        0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
            |opcode| SharpSm83Instruction::Undefined(Undefined::new(opcode))
        }
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
            |opcode| SharpSm83Instruction::LdR8Imm8(LdR8Imm8::new(opcode))
        }
        0x76 => |opcode| SharpSm83Instruction::Halt(Halt::new(opcode)),
        0x40..=0x7F => |opcode| SharpSm83Instruction::LdR8R8(LdR8R8::new(opcode)),
        0x01 | 0x11 | 0x21 | 0x31 => |opcode| SharpSm83Instruction::LdR16Imm16(LdR16Imm16::new(opcode)),
        0x08 => |opcode| SharpSm83Instruction::LdImm16Sp(LdImm16Sp::new(opcode)),
        0x02 | 0x12 | 0x22 | 0x32 => |opcode| SharpSm83Instruction::LdR16MemA(LdR16MemA::new(opcode)),
        0x0A | 0x1A | 0x2A | 0x3A => |opcode| SharpSm83Instruction::LdAR16Mem(LdAR16Mem::new(opcode)),
        0x03 | 0x13 | 0x23 | 0x33 => |opcode| SharpSm83Instruction::IncR16(IncR16::new(opcode)),
        0x0B | 0x1B | 0x2B | 0x3B => |opcode| SharpSm83Instruction::DecR16(DecR16::new(opcode)),
        0x09 | 0x19 | 0x29 | 0x39 => |opcode| SharpSm83Instruction::AddHlR16(AddHlR16::new(opcode)),
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => |opcode| SharpSm83Instruction::IncR8(IncR8::new(opcode)),
        0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => |opcode| SharpSm83Instruction::DecR8(DecR8::new(opcode)),
        0x07 => |opcode| SharpSm83Instruction::Rlca(Rlca::new(opcode)),
        0x0F => |opcode| SharpSm83Instruction::Rrca(Rrca::new(opcode)),
        0x17 => |opcode| SharpSm83Instruction::Rla(Rla::new(opcode)),
        0x1F => |opcode| SharpSm83Instruction::Rra(Rra::new(opcode)),
        0x27 => |opcode| SharpSm83Instruction::Daa(Daa::new(opcode)),
        0x2F => |opcode| SharpSm83Instruction::Cpl(Cpl::new(opcode)),
        0x37 => |opcode| SharpSm83Instruction::Scf(Scf::new(opcode)),
        0x3F => |opcode| SharpSm83Instruction::Ccf(Ccf::new(opcode)),
        0x18 => |opcode| SharpSm83Instruction::JrImm8(JrImm8::new(opcode)),
        0x20 | 0x28 | 0x30 | 0x38 => |opcode| SharpSm83Instruction::JrCondImm8(JrCondImm8::new(opcode)),
        0x10 => |opcode| SharpSm83Instruction::Stop(Stop::new(opcode)),
        0x80..=0x87 => |opcode| SharpSm83Instruction::AddAR8(AddAR8::new(opcode)),
        0x88..=0x8F => |opcode| SharpSm83Instruction::AdcAR8(AdcAR8::new(opcode)),
        0x90..=0x97 => |opcode| SharpSm83Instruction::SubAR8(SubAR8::new(opcode)),
        0x98..=0x9F => |opcode| SharpSm83Instruction::SbcAR8(SbcAR8::new(opcode)),
        0xA0..=0xA7 => |opcode| SharpSm83Instruction::AndAR8(AndAR8::new(opcode)),
        0xA8..=0xAF => |opcode| SharpSm83Instruction::XorAR8(XorAR8::new(opcode)),
        0xB0..=0xB7 => |opcode| SharpSm83Instruction::OrAR8(OrAR8::new(opcode)),
        0xB8..=0xBF => |opcode| SharpSm83Instruction::CpAR8(CpAR8::new(opcode)),
        0xC6 => |opcode| SharpSm83Instruction::AddAImm8(AddAImm8::new(opcode)),
        0xCE => |opcode| SharpSm83Instruction::AdcAImm8(AdcAImm8::new(opcode)),
        0xD6 => |opcode| SharpSm83Instruction::SubAImm8(SubAImm8::new(opcode)),
        0xDE => |opcode| SharpSm83Instruction::SbcAImm8(SbcAImm8::new(opcode)),
        0xE6 => |opcode| SharpSm83Instruction::AndAImm8(AndAImm8::new(opcode)),
        0xEE => |opcode| SharpSm83Instruction::XorAImm8(XorAImm8::new(opcode)),
        0xF6 => |opcode| SharpSm83Instruction::OrAImm8(OrAImm8::new(opcode)),
        0xFE => |opcode| SharpSm83Instruction::CpAImm8(CpAImm8::new(opcode)),
        0xC0 | 0xC8 | 0xD0 | 0xD8 => |opcode| SharpSm83Instruction::RetCond(RetCond::new(opcode)),
        0xC9 => |opcode| SharpSm83Instruction::Ret(Ret::new(opcode)),
        0xD9 => |opcode| SharpSm83Instruction::Reti(Reti::new(opcode)),
        0xC2 | 0xCA | 0xD2 | 0xDA => |opcode| SharpSm83Instruction::JpCondImm16(JpCondImm16::new(opcode)),
        0xC3 => |opcode| SharpSm83Instruction::JpImm16(JpImm16::new(opcode)),
        0xE9 => |opcode| SharpSm83Instruction::JpHl(JpHl::new(opcode)),
        0xC4 | 0xCC | 0xD4 | 0xDC => |opcode| SharpSm83Instruction::CallCondImm16(CallCondImm16::new(opcode)),
        0xCD => |opcode| SharpSm83Instruction::CallImm16(CallImm16::new(opcode)),
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            |opcode| SharpSm83Instruction::RstTgt3(RstTgt3::new(opcode))
        }
        0xC1 | 0xD1 | 0xE1 | 0xF1 => |opcode| SharpSm83Instruction::PopR16Stk(PopR16Stk::new(opcode)),
        0xC5 | 0xD5 | 0xE5 | 0xF5 => |opcode| SharpSm83Instruction::PushR16Stk(PushR16Stk::new(opcode)),
        0xE2 => |opcode| SharpSm83Instruction::LdhCMemA(LdhCMemA::new(opcode)),
        0xE0 => |opcode| SharpSm83Instruction::LdhImm8MemA(LdhImm8MemA::new(opcode)),
        0xEA => |opcode| SharpSm83Instruction::LdImm16MemA(LdImm16MemA::new(opcode)),
        0xF2 => |opcode| SharpSm83Instruction::LdhACMem(LdhACMem::new(opcode)),
        0xF0 => |opcode| SharpSm83Instruction::LdhAImm8Mem(LdhAImm8Mem::new(opcode)),
        0xFA => |opcode| SharpSm83Instruction::LdAImm16Mem(LdAImm16Mem::new(opcode)),
        0xE8 => |opcode| SharpSm83Instruction::AddSpSignedImm8(AddSpSignedImm8::new(opcode)),
        0xF8 => |opcode| SharpSm83Instruction::LdHlSpPlusSignedImm8(LdHlSpPlusSignedImm8::new(opcode)),
        0xF9 => |opcode| SharpSm83Instruction::LdSpHl(LdSpHl::new(opcode)),
        0xF3 => |opcode| SharpSm83Instruction::Di(Di::new(opcode)),
        0xFB => |opcode| SharpSm83Instruction::Ei(Ei::new(opcode)),
        0xCB => |opcode| SharpSm83Instruction::Prefix(Prefix::new(opcode)),
    }
}
