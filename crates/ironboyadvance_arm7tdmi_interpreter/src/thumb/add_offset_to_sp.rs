use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    cpu::{Arm7tdmiCpu, SP},
    memory::MemoryInterface,
    thumb::AddOffsetToSp,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for AddOffsetToSp {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = self.offset() * 4;
        let sp_value = cpu.register(SP);
        let value = match self.signed() {
            true => sp_value.wrapping_sub(offset as u32),
            false => sp_value.wrapping_add(offset as u32),
        };
        cpu.set_register(SP, value);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}
