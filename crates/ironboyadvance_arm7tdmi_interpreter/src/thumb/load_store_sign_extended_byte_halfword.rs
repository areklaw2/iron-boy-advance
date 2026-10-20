use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::LoadStoreSignExtendedByteHalfword,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for LoadStoreSignExtendedByteHalfword {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let ro_value = cpu.register(self.ro() as usize);
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(ro_value);

        let rd = self.rd() as usize;
        let signed = self.signed();
        let halfword = self.halfword();
        match (signed, halfword) {
            (false, false) => {
                let value = cpu.register(rd);
                cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
            }
            (false, true) => {
                let value = cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            (true, false) => {
                let value = cpu.load_signed_8(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            (true, true) => {
                let value = cpu.load_signed_16(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}
