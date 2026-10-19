use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    arm::SingleDataTransfer,
    barrel_shifter::*,
    cpu::{Arm7tdmiCpu, PC},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for SingleDataTransfer {
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
}
