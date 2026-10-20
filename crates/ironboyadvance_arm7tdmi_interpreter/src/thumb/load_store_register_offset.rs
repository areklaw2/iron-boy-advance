use crate::Execute;
use ironboyadvance_arm7tdmi::{CpuAction, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::LoadStoreRegisterOffset};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for LoadStoreRegisterOffset {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let ro_value = cpu.register(self.ro() as usize);
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(ro_value);

        let rd = self.rd() as usize;
        let byte = self.byte();
        let load = self.load();
        match (load, byte) {
            (false, false) => {
                let value = cpu.register(rd);
                cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
            }
            (false, true) => {
                let value = cpu.register(rd);
                cpu.store_8(address, value as u8, MemoryAccess::NonSequential as u8);
            }
            (true, false) => {
                let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            (true, true) => {
                let value = cpu.load_8(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}
