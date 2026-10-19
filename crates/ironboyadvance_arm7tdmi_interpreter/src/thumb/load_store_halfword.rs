use crate::Execute;
use ironboyadvance_arm7tdmi::{CpuAction, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::LoadStoreHalfword};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for LoadStoreHalfword {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = self.offset() * 2;
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(immediate as u32);
        let rd = self.rd() as usize;
        match self.load() {
            true => {
                let value = cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            false => {
                let value = cpu.register(rd);
                cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}
