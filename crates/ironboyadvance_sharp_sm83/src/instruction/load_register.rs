use ironboyadvance_common::bits::BitOps;

use crate::Register8;
use crate::cpu::Sm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy)]
pub(crate) enum LoadRegisterSource {
    Register(Register8),
    Immediate,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LoadRegister {
    destination: Register8,
    source: LoadRegisterSource,
}

impl LoadRegister {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            destination: Register8::from(opcode.bits(3..=5)),
            source: match opcode.bit(6) {
                true => LoadRegisterSource::Register(Register8::from(opcode.bits(0..=2))),
                false => LoadRegisterSource::Immediate,
            },
        }
    }
}

impl Instruction for LoadRegister {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) {
        let value = match self.source {
            LoadRegisterSource::Register(r8) => cpu.register_8(r8),
            LoadRegisterSource::Immediate => cpu.fetch_byte(),
        };
        cpu.set_register_8(self.destination, value);
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Sm83<I>) -> String {
        match self.source {
            LoadRegisterSource::Register(r8) => format!("LD {},{}", self.destination, r8),
            LoadRegisterSource::Immediate => format!("LD {},{:#04X}", self.destination, cpu.fetch_byte()),
        }
    }
}
