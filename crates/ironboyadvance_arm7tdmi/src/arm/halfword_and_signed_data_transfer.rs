use crate::{
    BitOps, CpuAction, Register,
    arm::arm_instruction,
    cpu::{Arm7tdmiCpu, Instruction, PC},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct HalfwordAndSignedDataTransfer {
    value: u32,
}

arm_instruction!(HalfwordAndSignedDataTransfer);

impl Instruction for HalfwordAndSignedDataTransfer {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rn = self.rn() as usize;

        let mut address = cpu.register(rn);
        let mut offset = match self.is_immediate() {
            true => self.immediate_hi() << 4 | self.immediate_lo(),
            false => cpu.register(self.rm() as usize),
        };

        if !self.add() {
            offset = (-(offset as i64)) as u32
        }

        let pre_index = self.pre_index();
        if pre_index {
            address = address.wrapping_add(offset)
        }

        let load = self.load();
        let write_back = self.write_back();
        let s = self.signed();
        let h = self.halfword();
        match load {
            true => match (s, h) {
                (false, false) => {}
                (false, true) => {
                    let value = cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8);
                    if write_back || !pre_index {
                        if rn != rd && rn == PC {
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                    }
                    cpu.idle_cycle();
                    cpu.set_register(rd, value);
                }
                (true, false) => {
                    let value = cpu.load_signed_8(address, MemoryAccess::NonSequential as u8);
                    if write_back || !pre_index {
                        if rn != rd && rn == PC {
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                    }
                    cpu.idle_cycle();
                    cpu.set_register(rd, value);
                }
                (true, true) => {
                    let value = cpu.load_signed_16(address, MemoryAccess::NonSequential as u8);
                    if write_back || !pre_index {
                        if rn != rd && rn == PC {
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                    }
                    cpu.idle_cycle();
                    cpu.set_register(rd, value);
                }
            },
            false => {
                let mut value = cpu.register(rd);
                if rd == PC {
                    value += 4;
                }
                match (s, h) {
                    (false, false) => {}
                    (false, true) => {
                        cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
                        if write_back || !pre_index {
                            if rn == PC {
                                cpu.pipeline_flush();
                            }
                            cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                        }
                    }
                    (true, false) => {
                        cpu.idle_cycle();
                        if write_back || !pre_index {
                            if rn == PC {
                                cpu.pipeline_flush();
                            }
                            cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                        }
                    }
                    (true, true) => {
                        cpu.idle_cycle();
                        if write_back || !pre_index {
                            if rn == PC {
                                cpu.pipeline_flush();
                            }
                            cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                        }
                    }
                };
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
        let add = if self.add() { "+" } else { "-" };
        let rn = self.rn();
        let rd = self.rd();
        let immediate = self.immediate_hi() << 4 | self.immediate_lo();
        let address = match rd as usize == 15 {
            true => format!("#{:08X}", immediate),
            false => {
                let rm = self.rm();
                let offset = match self.is_immediate() {
                    true => match immediate {
                        0 => "".into(),
                        _ => format!(",#{}{}", add, immediate),
                    },
                    false => format!(",{}{}", add, rm),
                };

                let write_back = if self.write_back() && !offset.is_empty() { "!" } else { "" };
                match pre_index {
                    true => format!("[{}{}]{}", rn, offset, write_back),
                    false => format!("[{}]{}", rn, offset),
                }
            }
        };

        let s = self.signed();
        let h = self.halfword();
        let sh = match (s, h) {
            (false, false) => "",
            (false, true) => "H",
            (true, false) => "SB",
            (true, true) => "SH",
        };

        match self.load() {
            true => format!("LDR{}{} {},{}", cond, sh, rd, address),
            false => format!("STR{}{} {},{}", cond, sh, rd, address),
        }
    }
}

impl HalfwordAndSignedDataTransfer {
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
        self.value.bit(22)
    }

    #[inline]
    pub fn immediate_hi(&self) -> u32 {
        self.value.bits(8..=11)
    }

    #[inline]
    pub fn immediate_lo(&self) -> u32 {
        self.value.bits(0..=3)
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
    pub fn write_back(&self) -> bool {
        self.value.bit(21)
    }

    #[inline]
    pub fn load(&self) -> bool {
        self.value.bit(20)
    }

    #[inline]
    pub fn signed(&self) -> bool {
        self.value.bit(6)
    }

    #[inline]
    pub fn halfword(&self) -> bool {
        self.value.bit(5)
    }
}
