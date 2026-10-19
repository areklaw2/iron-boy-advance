use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    arm::SingleDataSwap,
    cpu::{Arm7tdmiCpu, PC},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for SingleDataSwap {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rn = self.rn() as usize;
        let rm = self.rm() as usize;

        let address = cpu.register(rn);
        let mut source = cpu.register(rm);
        if rm == PC {
            source += 4;
        }

        let value: u32;
        match self.byte() {
            true => {
                value = cpu.load_8(address, MemoryAccess::NonSequential as u8);
                cpu.store_8(address, source as u8, MemoryAccess::NonSequential | MemoryAccess::Lock);
            }
            false => {
                value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.store_32(address, source, MemoryAccess::NonSequential | MemoryAccess::Lock);
            }
        };

        cpu.idle_cycle();
        cpu.set_register(rd, value);
        match rd == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }
}
