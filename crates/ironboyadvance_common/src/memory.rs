use std::ops::BitOr;

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
