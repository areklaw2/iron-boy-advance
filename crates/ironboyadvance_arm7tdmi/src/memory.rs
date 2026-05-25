use std::ops::BitOr;

use crate::{CpuState, bits::SignExtend, cpu::Arm7tdmiCpu};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryAccess {
    NonSequential = 0b0,
    Sequential = 0b1,
    Instruction = 0b10,
    Dma = 0b100,
    Lock = 0b1000,
}

impl MemoryAccess {
    pub fn is_set(self, access_pattern: u8) -> bool {
        access_pattern & self as u8 != 0
    }
}

impl BitOr for MemoryAccess {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryAccessWidth {
    Byte,
    HalfWord,
    Word,
}

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

pub trait SystemMemoryAccess {
    fn read_8(&self, address: u32) -> u8;

    fn read_16(&self, address: u32) -> u16 {
        let byte1 = self.read_8(address) as u16;
        let byte2 = self.read_8(address + 1) as u16;
        byte2 << 8 | byte1
    }

    fn read_32(&self, address: u32) -> u32 {
        let half_word1 = self.read_16(address) as u32;
        let half_word2 = self.read_16(address + 2) as u32;
        half_word2 << 16 | half_word1
    }

    fn write_8(&mut self, address: u32, value: u8);

    fn write_16(&mut self, address: u32, value: u16) {
        let byte1 = (value & 0xFF) as u8;
        let byte2 = (value >> 8) as u8;
        self.write_8(address, byte1);
        self.write_8(address + 1, byte2);
    }

    fn write_32(&mut self, address: u32, value: u32) {
        let half_word1 = (value & 0xFFFF) as u16;
        let half_word2 = (value >> 16) as u16;
        self.write_16(address, half_word1);
        self.write_16(address + 2, half_word2);
    }
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
