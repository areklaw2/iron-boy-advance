use crate::BitOps;

use crate::{
    CpuAction, Register,
    arm::arm_instruction,
    barrel_shifter::{ShiftType, asr, lsl, lsr, ror},
    cpu::{Instruction, Arm7tdmiCpu, PC},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct SingleDataTransfer {
    value: u32,
}

arm_instruction!(SingleDataTransfer);

impl Instruction for SingleDataTransfer {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rn = self.rn() as usize;

        let mut address = cpu.register(rn);
        let mut offset = match self.is_immediate() {
            true => self.immediate(),
            false => {
                let rm_value = cpu.register(self.rm() as usize);
                let shift_amount = self.shift_amount();
                let mut carry = cpu.cpsr().carry();
                match self.shift_type() {
                    ShiftType::LSL => lsl(rm_value, shift_amount, &mut carry),
                    ShiftType::LSR => lsr(rm_value, shift_amount, &mut carry, true),
                    ShiftType::ASR => asr(rm_value, shift_amount, &mut carry, true),
                    ShiftType::ROR => ror(rm_value, shift_amount, &mut carry, true),
                }
            }
        };

        if !self.add() {
            offset = (-(offset as i64)) as u32
        }

        let pre_index = self.pre_index();
        if pre_index {
            address = address.wrapping_add(offset)
        }

        let load = self.load();
        let byte = self.byte();
        let write_back = self.write_back();
        match load {
            true => {
                let value = match byte {
                    true => cpu.load_8(address, MemoryAccess::NonSequential as u8),
                    false => cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8),
                };
                if write_back || !pre_index {
                    if rn != rd && rn == PC {
                        cpu.pipeline_flush();
                    }
                    cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                }
                cpu.idle_cycle();
                cpu.set_register(rd, value);
            }
            false => {
                let mut value = cpu.register(rd);
                if rd == PC {
                    value += 4;
                }
                match byte {
                    true => cpu.store_8(address, value as u8, MemoryAccess::NonSequential as u8),
                    false => cpu.store_32(address, value, MemoryAccess::NonSequential as u8),
                };
                if write_back || !pre_index {
                    if rn == PC {
                        cpu.pipeline_flush();
                    }
                    cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                }
            }
        }

        match load && rd == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let pre_index = self.pre_index();
        let t = if pre_index { "" } else { "T" };
        let add = if self.add() { "+" } else { "-" };
        let byte = if self.byte() { "B" } else { "" };
        let rn = self.rn();
        let rd = self.rd();
        let immediate = self.immediate();
        let address = match rd as usize == 15 {
            true => format!("#{:08X}", immediate),
            false => {
                let offset = match self.is_immediate() {
                    true => match immediate {
                        0 => "".into(),
                        _ => format!(",#{}{}", add, immediate),
                    },
                    false => {
                        let rm = self.rm();
                        let shift_type = self.shift_type();
                        format!(",{}{},{} #{}", add, rm, shift_type, self.shift_amount())
                    }
                };

                let write_back = if self.write_back() && !offset.is_empty() { "!" } else { "" };
                match pre_index {
                    true => format!("[{}{}]{}", rn, offset, write_back),
                    false => format!("[{}]{}", rn, offset),
                }
            }
        };

        match self.load() {
            true => format!("LDR{}{}{} {},{}", cond, byte, t, rd, address),
            false => format!("STR{}{}{} {},{}", cond, byte, t, rd, address),
        }
    }
}

impl SingleDataTransfer {
    #[inline]
    pub fn rn(&self) -> Register {
        self.value.bits(16..=19).into()
    }

    #[inline]
    pub fn rd(&self) -> Register {
        self.value.bits(12..=15).into()
    }

    #[inline]
    pub fn rm(&self) -> Register {
        self.value.bits(0..=3).into()
    }

    #[inline]
    pub fn is_immediate(&self) -> bool {
        !self.value.bit(25)
    }

    #[inline]
    pub fn shift_amount(&self) -> u32 {
        self.value.bits(7..=11)
    }

    #[inline]
    pub fn shift_type(&self) -> ShiftType {
        self.value.bits(5..=6).into()
    }

    #[inline]
    pub fn immediate(&self) -> u32 {
        self.value.bits(0..=11)
    }

    #[inline]
    pub fn pre_index(&self) -> bool {
        self.value.bit(24)
    }

    #[inline]
    pub fn add(&self) -> bool {
        self.value.bit(23)
    }

    #[inline]
    pub fn byte(&self) -> bool {
        self.value.bit(22)
    }

    #[inline]
    pub fn write_back(&self) -> bool {
        self.value.bit(21)
    }

    #[inline]
    pub fn load(&self) -> bool {
        self.value.bit(20)
    }
}
