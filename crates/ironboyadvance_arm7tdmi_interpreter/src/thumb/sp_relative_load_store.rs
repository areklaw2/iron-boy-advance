use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    cpu::{Arm7tdmiCpu, SP},
    memory::MemoryInterface,
    thumb::SpRelativeLoadStore,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for SpRelativeLoadStore {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = self.offset() * 4;
        let sp_value = cpu.register(SP);
        let address = sp_value.wrapping_add(immediate as u32);
        let rd = self.rd() as usize;
        match self.load() {
            true => {
                let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            false => {
                let value = cpu.register(rd);
                cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}
