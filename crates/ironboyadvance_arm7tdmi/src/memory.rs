use ironboyadvance_common::bits::SignExtend;

use crate::{CpuState, cpu::Arm7tdmiCpu};

#[derive(Debug, Copy, Clone)]
pub struct CpuContext {
    pub pc: u32,
    pub cpu_state: CpuState,
    pub pipeline: [u32; 2],
}

impl Default for CpuContext {
    fn default() -> Self {
        Self {
            pc: 0,
            cpu_state: CpuState::Arm,
            pipeline: [0; 2],
        }
    }
}

pub trait MemoryInterface {
    fn load_8(&mut self, address: u32, access_pattern: u8) -> u32;

    fn load_16(&mut self, address: u32, access_pattern: u8) -> u32;

    fn load_32(&mut self, address: u32, access_pattern: u8) -> u32;

    fn store_8(&mut self, address: u32, value: u8, access_pattern: u8);

    fn store_16(&mut self, address: u32, value: u16, access_pattern: u8);

    fn store_32(&mut self, address: u32, value: u32, access_pattern: u8);

    fn idle_cycle(&mut self);

    fn cpu_context_mut(&mut self) -> &mut CpuContext;
}

impl<I: MemoryInterface> Arm7tdmiCpu<I> {
    pub(crate) fn load_signed_8(&mut self, address: u32, access_pattern: u8) -> u32 {
        self.load_8(address, access_pattern).sign_extend(8) as u32
    }

    pub(crate) fn load_signed_16(&mut self, address: u32, access_pattern: u8) -> u32 {
        let value = self.load_16(address, access_pattern);
        match address & 0x1 != 0 {
            true => (value >> 8).sign_extend(8) as u32,
            false => value.sign_extend(16) as u32,
        }
    }

    pub(crate) fn load_rotated_16(&mut self, address: u32, access_pattern: u8) -> u32 {
        let value = self.load_16(address, access_pattern);
        match address & 0x1 != 0 {
            true => value.rotate_right(8),
            false => value,
        }
    }

    pub(crate) fn load_rotated_32(&mut self, address: u32, access_pattern: u8) -> u32 {
        let value = self.load_32(address, access_pattern);
        let rotation = (address & 0x3) << 3;
        value >> rotation | value.wrapping_shl(32 - rotation)
    }
}
