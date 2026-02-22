use crate::BitOps;

use crate::{
    CpuAction, LoRegister,
    cpu::Arm7tdmiCpu,
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct LoadStoreSignExtendedByteHalfword {
    value: u16,
}

thumb_instruction!(LoadStoreSignExtendedByteHalfword);

impl LoadStoreSignExtendedByteHalfword {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
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

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let ro = self.ro();
        let rb = self.rb();
        let rd = self.rd();
        let signed = self.signed();
        let halfword = self.halfword();
        match (signed, halfword) {
            (false, false) => format!("STRH {}, [{},{}]", rd, rb, ro),
            (false, true) => format!("LDRH {}, [{},{}]", rd, rb, ro),
            (true, false) => format!("LDSB {}, [{},{}]", rd, rb, ro),
            (true, true) => format!("LDSH {}, [{},{}]", rd, rb, ro),
        }
    }

    #[inline]
    pub fn rd(&self) -> LoRegister {
        self.value.bits(0..=2).into()
    }

    #[inline]
    pub fn rb(&self) -> LoRegister {
        self.value.bits(3..=5).into()
    }

    #[inline]
    pub fn ro(&self) -> LoRegister {
        self.value.bits(6..=8).into()
    }

    #[inline]
    pub fn signed(&self) -> bool {
        self.value.bit(10)
    }

    #[inline]
    pub fn halfword(&self) -> bool {
        self.value.bit(11)
    }
}
