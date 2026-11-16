use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    arm::HalfwordAndSignedDataTransfer,
    cpu::{Arm7tdmiCpu, PC},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for HalfwordAndSignedDataTransfer {
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
            address = address.wrapping_add(offset);
        }

        let load = self.load();
        let updates_base = self.write_back() || !pre_index;
        let s = self.signed();
        let h = self.halfword();
        match load {
            true => {
                let value = match (s, h) {
                    (false, false) => None,
                    (false, true) => Some(cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8)),
                    (true, false) => Some(cpu.load_signed_8(address, MemoryAccess::NonSequential as u8)),
                    (true, true) => Some(cpu.load_signed_16(address, MemoryAccess::NonSequential as u8)),
                };

                if let Some(value) = value {
                    if updates_base {
                        write_back(cpu, rn, offset, rn != rd && rn == PC);
                    }
                    cpu.idle_cycle();
                    cpu.set_register(rd, value);
                }
            }
            false => {
                let mut value = cpu.register(rd);
                if rd == PC {
                    value += 4;
                }
                match (s, h) {
                    (false, false) => {}
                    (false, true) => {
                        cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
                        if updates_base {
                            write_back(cpu, rn, offset, rn == PC);
                        }
                    }
                    (true, _) => {
                        cpu.idle_cycle();
                        if updates_base {
                            write_back(cpu, rn, offset, rn == PC);
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
}

fn write_back<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, rn: usize, offset: u32, flush_first: bool) {
    if flush_first {
        cpu.pipeline_flush();
    }
    cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
}
