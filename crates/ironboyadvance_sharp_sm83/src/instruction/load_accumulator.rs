use ironboyadvance_common::bits::BitOps;

use crate::R16Memory;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum AccumulatorAddressing {
    Indirect(R16Memory),
    Direct,
    DirectHighPage,
    IndirectHighPageC,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LoadAccumulator {
    addressing: AccumulatorAddressing,
    is_load: bool,
}

impl LoadAccumulator {
    pub(crate) fn new(opcode: u8) -> Self {
        match opcode.bits(6..=7) {
            0b00 => Self {
                addressing: AccumulatorAddressing::Indirect(R16Memory::from(opcode.bits(4..=5))),
                is_load: opcode.bit(3),
            },
            _ => Self {
                addressing: match opcode.bits(0..=3) {
                    0b0000 => AccumulatorAddressing::DirectHighPage,
                    0b0010 => AccumulatorAddressing::IndirectHighPageC,
                    _ => AccumulatorAddressing::Direct,
                },
                is_load: opcode.bit(4),
            },
        }
    }
}

impl Instruction for LoadAccumulator {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        let address = match self.addressing {
            AccumulatorAddressing::Indirect(r16_memory) => cpu.register_16_memory(r16_memory),
            AccumulatorAddressing::Direct => cpu.fetch_word(),
            AccumulatorAddressing::DirectHighPage => 0xFF00 | cpu.fetch_byte() as u16,
            AccumulatorAddressing::IndirectHighPageC => 0xFF00 | cpu.registers().c() as u16,
        };

        match self.is_load {
            true => {
                let value = cpu.bus().load_8(address);
                cpu.registers_mut().set_a(value);
            }
            false => {
                let value = cpu.registers().a();
                cpu.bus_mut().store_8(address, value);
            }
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        let location = match self.addressing {
            AccumulatorAddressing::Indirect(r16_memory) => format!("[{}]", r16_memory),
            AccumulatorAddressing::Direct => format!("[{:#04X}]", cpu.fetch_word()),
            AccumulatorAddressing::DirectHighPage => format!("[FF00+{:#04X}]", cpu.fetch_byte()),
            AccumulatorAddressing::IndirectHighPageC => "[FF00+C]".to_string(),
        };

        match self.is_load {
            true => format!("LD A,{}", location),
            false => format!("LD {},A", location),
        }
    }
}
