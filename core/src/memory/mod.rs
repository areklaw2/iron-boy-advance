pub mod system_bus;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryAccess {
    Sequential,
    NonSequential,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryAccessWidth {
    Byte,
    HalfWord,
    Word,
}

pub trait MemoryInterface {
    fn load_8(&mut self, address: u32, access: MemoryAccess) -> u8;

    fn load_16(&mut self, address: u32, access: MemoryAccess) -> u16;

    fn load_32(&mut self, address: u32, access: MemoryAccess) -> u32;

    fn store_8(&mut self, address: u32, value: u8, access: MemoryAccess);

    fn store_16(&mut self, address: u32, value: u16, access: MemoryAccess);

    fn store_32(&mut self, address: u32, value: u32, access: MemoryAccess);
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
