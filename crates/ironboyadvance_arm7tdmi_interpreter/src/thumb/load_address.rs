use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    cpu::{Arm7tdmiCpu, SP},
    memory::MemoryInterface,
    thumb::LoadAddress,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for LoadAddress {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let offset = self.offset() * 4;
        let value = match self.sp() {
            true => cpu.register(SP).wrapping_add(offset as u32),
            false => (cpu.pc() & !0b10).wrapping_add(offset as u32),
        };
        cpu.set_register(rd, value);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}
