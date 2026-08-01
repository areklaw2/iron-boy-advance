use crate::alu;
use crate::cpu::SharpSm83;
use crate::instruction::Instruction;
use crate::memory::MemoryInterface;

const LD_IMM16_SP: u8 = 0x08;
const LD_HL_SP_PLUS_SIGNED_IMM8: u8 = 0xF8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum StackPointerLoadOpcode {
    DirectFromStackPointer,
    HlFromAdjustedStackPointer,
    StackPointerFromHl,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StackPointerLoad {
    opcode: StackPointerLoadOpcode,
}

impl StackPointerLoad {
    pub(crate) fn new(opcode: u8) -> Self {
        Self {
            opcode: match opcode {
                LD_IMM16_SP => StackPointerLoadOpcode::DirectFromStackPointer,
                LD_HL_SP_PLUS_SIGNED_IMM8 => StackPointerLoadOpcode::HlFromAdjustedStackPointer,
                _ => StackPointerLoadOpcode::StackPointerFromHl,
            },
        }
    }
}

impl Instruction for StackPointerLoad {
    fn execute<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) {
        match self.opcode {
            StackPointerLoadOpcode::DirectFromStackPointer => {
                let address = cpu.fetch_word();
                let sp = cpu.registers().sp();
                cpu.bus_mut().store_16(address, sp);
            }
            StackPointerLoadOpcode::HlFromAdjustedStackPointer => {
                let offset = cpu.fetch_byte() as i8 as i16 as u16;
                cpu.bus_mut().idle_cycle();
                let result = alu::add_offset_to_stack_pointer(cpu, offset);
                cpu.registers_mut().set_hl(result);
            }
            StackPointerLoadOpcode::StackPointerFromHl => {
                let hl = cpu.registers().hl();
                cpu.registers_mut().set_sp(hl);
                cpu.bus_mut().idle_cycle();
            }
        }
    }

    fn disassemble<I: MemoryInterface>(&self, cpu: &mut SharpSm83<I>) -> String {
        match self.opcode {
            StackPointerLoadOpcode::DirectFromStackPointer => format!("LD {:#04X},SP", cpu.fetch_word()),
            StackPointerLoadOpcode::HlFromAdjustedStackPointer => format!("LD HL,SP+{:#04X}", cpu.fetch_byte()),
            StackPointerLoadOpcode::StackPointerFromHl => "LD SP,HL".to_string(),
        }
    }
}
