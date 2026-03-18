use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct LoadStoreSignExtendedByteHalfword {
    rd: LoRegister,
    rb: LoRegister,
    ro: LoRegister,
    signed: bool,
    halfword: bool,
}

impl LoadStoreSignExtendedByteHalfword {
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rb: value.bits(3..=5).into(),
            ro: value.bits(6..=8).into(),
            signed: value.bit(10),
            halfword: value.bit(11),
        }
    }
}

impl Instruction for LoadStoreSignExtendedByteHalfword {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let ro_value = cpu.register(self.ro as usize);
        let rb_value = cpu.register(self.rb as usize);
        let address = rb_value.wrapping_add(ro_value);

        let rd = self.rd as usize;
        let signed = self.signed;
        let halfword = self.halfword;
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

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let ro = self.ro;
        let rb = self.rb;
        let rd = self.rd;
        let signed = self.signed;
        let halfword = self.halfword;
        match (signed, halfword) {
            (false, false) => format!("STRH {}, [{},{}]", rd, rb, ro),
            (false, true) => format!("LDRH {}, [{},{}]", rd, rb, ro),
            (true, false) => format!("LDSB {}, [{},{}]", rd, rb, ro),
            (true, true) => format!("LDSH {}, [{},{}]", rd, rb, ro),
        }
    }
}
