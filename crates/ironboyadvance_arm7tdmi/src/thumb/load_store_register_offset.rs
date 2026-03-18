use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct LoadStoreRegisterOffset {
    rd: LoRegister,
    rb: LoRegister,
    ro: LoRegister,
    byte: bool,
    load: bool,
}

impl LoadStoreRegisterOffset {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rb: value.bits(3..=5).into(),
            ro: value.bits(6..=8).into(),
            byte: value.bit(10),
            load: value.bit(11),
        }
    }
}

impl Instruction for LoadStoreRegisterOffset {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let ro_value = cpu.register(self.ro as usize);
        let rb_value = cpu.register(self.rb as usize);
        let address = rb_value.wrapping_add(ro_value);

        let rd = self.rd as usize;
        let byte = self.byte;
        let load = self.load;
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

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let byte = if self.byte { "B" } else { "" };
        let ro = self.ro;
        let rb = self.rb;
        let rd = self.rd;

        match self.load {
            true => format!("LDR{} {}, [{},{}]", byte, rd, rb, ro),
            false => format!("STR{} {}, [{},{}]", byte, rd, rb, ro),
        }
    }
}
