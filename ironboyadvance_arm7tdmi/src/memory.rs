use std::ops::BitOr;

use crate::cpu::Arm7tdmiCpu;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryAccess {
    Nonsequential = 0b0,
    Sequential = 0b1,
    Instruction = 0b10,
    Dma = 0b100,
    Lock = 0b1000,
}

impl BitOr for MemoryAccess {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

pub fn decompose_access_pattern(access_pattern: u8) -> Vec<MemoryAccess> {
    let mut decomposition = Vec::new();
    match access_pattern & MemoryAccess::Sequential as u8 != 0 {
        true => decomposition.push(MemoryAccess::Sequential),
        false => decomposition.push(MemoryAccess::Nonsequential),
    };

    if access_pattern & MemoryAccess::Instruction as u8 != 0 {
        decomposition.push(MemoryAccess::Instruction);
    }

    if access_pattern & MemoryAccess::Dma as u8 != 0 {
        decomposition.push(MemoryAccess::Dma);
    }

    if access_pattern & MemoryAccess::Lock as u8 != 0 {
        decomposition.push(MemoryAccess::Lock);
    }
    decomposition
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryAccessWidth {
    Byte,
    HalfWord,
    Word,
}

pub trait MemoryInterface {
    fn load_8(&mut self, address: u32, access_pattern: u8) -> u32;

    fn load_16(&mut self, address: u32, access_pattern: u8) -> u32;

    fn load_32(&mut self, address: u32, access_pattern: u8) -> u32;

    fn store_8(&mut self, address: u32, value: u8, access_pattern: u8);

    fn store_16(&mut self, address: u32, value: u16, access_pattern: u8);

    fn store_32(&mut self, address: u32, value: u32, access_pattern: u8);

    fn idle_cycle(&mut self);
}

pub trait IoMemoryAccess {
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
    pub fn load_signed_8(&mut self, address: u32, access_pattern: u8) -> u32 {
        self.load_8(address, access_pattern) as i8 as i32 as u32
    }

    pub fn load_signed_16(&mut self, address: u32, access_pattern: u8) -> u32 {
        println!("address: {}", address);

        match address & 0x1 != 0 {
            true => self.load_8(address, access_pattern) as i8 as i32 as u32,
            false => self.load_16(address, access_pattern) as i16 as i32 as u32,
        }
    }

    pub fn load_rotated_16(&mut self, address: u32, access_pattern: u8) -> u32 {
        let value = self.load_16(address, access_pattern);
        match address & 0x1 != 0 {
            true => value >> 8 | value << 24,
            false => value,
        }
    }

    pub fn load_rotated_32(&mut self, address: u32, access_pattern: u8) -> u32 {
        let value = self.load_32(address, access_pattern);
        let rotation = (address & 0x3) << 3;
        value >> rotation | value.wrapping_shl(32 - rotation)
    }
}
