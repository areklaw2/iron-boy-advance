use crate::{
    CpuAction, Exception,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
};
use ironboyadvance_common::{bits::BitOps, memory::MemoryAccess};

#[derive(Debug, Clone, Copy)]
pub struct SoftwareInterrupt {
    offset: u16,
}

impl SoftwareInterrupt {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            offset: value.bits(0..=7),
        }
    }
}

impl Instruction for SoftwareInterrupt {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match !cpu.bios_loaded() && cpu.bios_call(self.offset as u32) {
            true => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
            false => {
                cpu.exception(Exception::SoftwareInterrupt);
                CpuAction::PipelineFlush
            }
        }
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let offset = self.offset;
        format!("SWI #{}", offset)
    }
}
