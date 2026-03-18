use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction, PC},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct PcRelativeLoad {
    rd: LoRegister,
    offset: u16,
}

impl PcRelativeLoad {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(8..=10).into(),
            offset: value.bits(0..=7),
        }
    }
}

impl Instruction for PcRelativeLoad {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = self.offset;
        let address = (cpu.register(PC) & !0x2).wrapping_add((offset << 2) as u32);
        let value = cpu.load_32(address, MemoryAccess::NonSequential as u8);
        cpu.set_register(self.rd as usize, value);
        cpu.idle_cycle();
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd;
        let offset = self.offset;
        format!("LDR {},[PC, #{}]", rd, offset)
    }
}
