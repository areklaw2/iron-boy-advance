use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    cpu::{Arm7tdmiCpu, PC},
    memory::MemoryInterface,
    thumb::PcRelativeLoad,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for PcRelativeLoad {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = self.offset();
        let address = (cpu.register(PC) & !0x2).wrapping_add((offset << 2) as u32);
        let value = cpu.load_32(address, MemoryAccess::NonSequential as u8);
        cpu.set_register(self.rd() as usize, value);
        cpu.idle_cycle();
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}
