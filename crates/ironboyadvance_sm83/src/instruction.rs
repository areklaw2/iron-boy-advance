use crate::cpu::Sm83;
use crate::instruction::accumulator_operations::AccumulatorOperations;
use crate::instruction::alu_8::Alu8;
use crate::instruction::alu_16::Alu16;
use crate::instruction::call::Call;
use crate::instruction::halt::Halt;
use crate::instruction::increment_decrement::IncrementDecrement;
use crate::instruction::interrupt_enable::InterruptEnable;
use crate::instruction::jump::Jump;
use crate::instruction::load_accumulator::LoadAccumulator;
use crate::instruction::load_register::LoadRegister;
use crate::instruction::load_register_16::LoadRegister16;
use crate::instruction::nop::Nop;
use crate::instruction::prefix::Prefix;
use crate::instruction::relative_jump::RelativeJump;
use crate::instruction::restart::Restart;
use crate::instruction::ret::Ret;
use crate::instruction::stack_operations::StackOperations;
use crate::instruction::stack_pointer_load::StackPointerLoad;
use crate::instruction::stop::Stop;
use crate::instruction::undefined::Undefined;
use crate::memory::MemoryInterface;

mod accumulator_operations;
mod alu_16;
mod alu_8;
mod call;
mod cb;
mod halt;
mod increment_decrement;
mod interrupt_enable;
mod jump;
mod load_accumulator;
mod load_register;
mod load_register_16;
mod nop;
mod prefix;
mod relative_jump;
mod restart;
mod ret;
mod stack_operations;
mod stack_pointer_load;
mod stop;
mod undefined;

pub(crate) trait Instruction {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>);
    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) -> String;
}

pub(crate) type SharpSm83InstructionFactory = fn(u8) -> SharpSm83Instruction;

#[derive(Debug, Clone, Copy)]
pub(crate) enum SharpSm83Instruction {
    Nop(Nop),
    Undefined(Undefined),
    Stop(Stop),
    Halt(Halt),
    Prefix(Prefix),
    LoadRegister(LoadRegister),
    LoadRegister16(LoadRegister16),
    LoadAccumulator(LoadAccumulator),
    Alu16(Alu16),
    IncrementDecrement(IncrementDecrement),
    AccumulatorOperations(AccumulatorOperations),
    Alu8(Alu8),
    RelativeJump(RelativeJump),
    Jump(Jump),
    Call(Call),
    Ret(Ret),
    Restart(Restart),
    StackOperations(StackOperations),
    StackPointerLoad(StackPointerLoad),
    InterruptEnable(InterruptEnable),
}

impl Instruction for SharpSm83Instruction {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        match self {
            Self::Nop(i) => i.execute(cpu),
            Self::Undefined(i) => i.execute(cpu),
            Self::Stop(i) => i.execute(cpu),
            Self::Halt(i) => i.execute(cpu),
            Self::Prefix(i) => i.execute(cpu),
            Self::LoadRegister(i) => i.execute(cpu),
            Self::LoadRegister16(i) => i.execute(cpu),
            Self::LoadAccumulator(i) => i.execute(cpu),
            Self::Alu16(i) => i.execute(cpu),
            Self::IncrementDecrement(i) => i.execute(cpu),
            Self::AccumulatorOperations(i) => i.execute(cpu),
            Self::Alu8(i) => i.execute(cpu),
            Self::RelativeJump(i) => i.execute(cpu),
            Self::Jump(i) => i.execute(cpu),
            Self::Call(i) => i.execute(cpu),
            Self::Ret(i) => i.execute(cpu),
            Self::Restart(i) => i.execute(cpu),
            Self::StackOperations(i) => i.execute(cpu),
            Self::StackPointerLoad(i) => i.execute(cpu),
            Self::InterruptEnable(i) => i.execute(cpu),
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) -> String {
        match self {
            Self::Nop(i) => i.disassemble(cpu),
            Self::Undefined(i) => i.disassemble(cpu),
            Self::Stop(i) => i.disassemble(cpu),
            Self::Halt(i) => i.disassemble(cpu),
            Self::Prefix(i) => i.disassemble(cpu),
            Self::LoadRegister(i) => i.disassemble(cpu),
            Self::LoadRegister16(i) => i.disassemble(cpu),
            Self::LoadAccumulator(i) => i.disassemble(cpu),
            Self::Alu16(i) => i.disassemble(cpu),
            Self::IncrementDecrement(i) => i.disassemble(cpu),
            Self::AccumulatorOperations(i) => i.disassemble(cpu),
            Self::Alu8(i) => i.disassemble(cpu),
            Self::RelativeJump(i) => i.disassemble(cpu),
            Self::Jump(i) => i.disassemble(cpu),
            Self::Call(i) => i.disassemble(cpu),
            Self::Ret(i) => i.disassemble(cpu),
            Self::Restart(i) => i.disassemble(cpu),
            Self::StackOperations(i) => i.disassemble(cpu),
            Self::StackPointerLoad(i) => i.disassemble(cpu),
            Self::InterruptEnable(i) => i.disassemble(cpu),
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
        0x10 => |opcode| SharpSm83Instruction::Stop(Stop::new(opcode)),
        0x01 | 0x11 | 0x21 | 0x31 => |opcode| SharpSm83Instruction::LoadRegister16(LoadRegister16::new(opcode)),
        0x02 | 0x12 | 0x22 | 0x32 | 0x0A | 0x1A | 0x2A | 0x3A => {
            |opcode| SharpSm83Instruction::LoadAccumulator(LoadAccumulator::new(opcode))
        }
        0x03 | 0x13 | 0x23 | 0x33 | 0x0B | 0x1B | 0x2B | 0x3B | 0x09 | 0x19 | 0x29 | 0x39 | 0xE8 => {
            |opcode| SharpSm83Instruction::Alu16(Alu16::new(opcode))
        }
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C | 0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
            |opcode| SharpSm83Instruction::IncrementDecrement(IncrementDecrement::new(opcode))
        }
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
            |opcode| SharpSm83Instruction::LoadRegister(LoadRegister::new(opcode))
        }
        0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F => {
            |opcode| SharpSm83Instruction::AccumulatorOperations(AccumulatorOperations::new(opcode))
        }
        0x18 | 0x20 | 0x28 | 0x30 | 0x38 => |opcode| SharpSm83Instruction::RelativeJump(RelativeJump::new(opcode)),
        0x76 => |opcode| SharpSm83Instruction::Halt(Halt::new(opcode)),
        0x40..=0x7F => |opcode| SharpSm83Instruction::LoadRegister(LoadRegister::new(opcode)),
        0x80..=0xBF => |opcode| SharpSm83Instruction::Alu8(Alu8::new(opcode)),
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => |opcode| SharpSm83Instruction::Alu8(Alu8::new(opcode)),

        0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xC9 | 0xD9 => |opcode| SharpSm83Instruction::Ret(Ret::new(opcode)),
        0xC2 | 0xCA | 0xD2 | 0xDA | 0xC3 | 0xE9 => |opcode| SharpSm83Instruction::Jump(Jump::new(opcode)),
        0xC4 | 0xCC | 0xD4 | 0xDC | 0xCD => |opcode| SharpSm83Instruction::Call(Call::new(opcode)),
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            |opcode| SharpSm83Instruction::Restart(Restart::new(opcode))
        }
        0xC1 | 0xD1 | 0xE1 | 0xF1 | 0xC5 | 0xD5 | 0xE5 | 0xF5 => {
            |opcode| SharpSm83Instruction::StackOperations(StackOperations::new(opcode))
        }
        0xCB => |opcode| SharpSm83Instruction::Prefix(Prefix::new(opcode)),
        0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xFA => {
            |opcode| SharpSm83Instruction::LoadAccumulator(LoadAccumulator::new(opcode))
        }
        0x08 | 0xF8 | 0xF9 => |opcode| SharpSm83Instruction::StackPointerLoad(StackPointerLoad::new(opcode)),
        0xF3 | 0xFB => |opcode| SharpSm83Instruction::InterruptEnable(InterruptEnable::new(opcode)),
        0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
            |opcode| SharpSm83Instruction::Undefined(Undefined::new(opcode))
        }
    }
}
