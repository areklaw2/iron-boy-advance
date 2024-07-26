pub enum MemoryAccess {
    Sequential,
    NonSequential,
}

pub trait MemoryInterface {
    fn read_byte(&self, address: u32) -> u8;

    fn read_half_word(&self, address: u32) -> u16 {
        let byte1 = self.read_byte(address) as u16;
        let byte2 = self.read_byte(address + 1) as u16;
        byte2 << 8 | byte1
    }

    fn read_word(&self, address: u32) -> u32 {
        let half_word1 = self.read_half_word(address) as u32;
        let half_word2 = self.read_half_word(address + 2) as u32;
        half_word2 << 16 | half_word1
    }

    fn write_byte(&mut self, address: u32, value: u8);

    fn write_half_word(&mut self, address: u32, value: u16) {
        let byte1 = (value & 0xFF) as u8;
        let byte2 = (value >> 8) as u8;
        self.write_byte(address, byte1);
        self.write_byte(address + 1, byte2);
    }

    fn write_word(&mut self, address: u32, value: u32) {
        let half_word1 = (value & 0xFFFF) as u16;
        let half_word2 = (value >> 16) as u16;
        self.write_half_word(address, half_word1);
        self.write_half_word(address + 2, half_word2);
    }
}

pub struct Bus {
    data: Vec<u8>,
}

impl MemoryInterface for Bus {
    fn read_byte(&self, address: u32) -> u8 {
        self.data[address as usize]
    }

    fn write_byte(&mut self, address: u32, value: u8) {
        self.data[address as usize] = value;
    }
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            data: vec![0; 0xFFFFFFFF],
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Bus {
            data: vec![0; 0xFFFFFFFF],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::MemoryInterface;

    use super::Bus;

    #[test]
    fn bus_read_byte() {
        let bus = Bus::default();
        assert_eq!(bus.read_byte(0x11111111), 0);
    }

    #[test]
    fn bus_write_byte() {
        let mut bus = Bus::default();
        bus.write_byte(0x11111111, 0x55);
        assert_eq!(bus.read_byte(0x11111111), 0x55);
    }

    #[test]
    fn bus_read_half_word() {
        let mut bus = Bus::default();
        bus.write_byte(0x11111111, 0x55);
        bus.write_byte(0x11111112, 0x3F);
        assert_eq!(bus.read_half_word(0x11111111), 0x3F55);
    }

    #[test]
    fn bus_write_half_word() {
        let mut bus = Bus::default();
        bus.write_half_word(0x11111111, 0x3F55);
        assert_eq!(bus.read_half_word(0x11111111), 0x3F55);
    }

    #[test]
    fn bus_read_word() {
        let mut bus = Bus::default();
        bus.write_half_word(0x11111111, 0x3F55);
        bus.write_half_word(0x11111113, 0xCC69);
        assert_eq!(bus.read_word(0x11111111), 0xCC693F55);
    }

    #[test]
    fn bus_write_word() {
        let mut bus = Bus::default();
        bus.write_word(0x11111111, 0xCC693F55);
        assert_eq!(bus.read_word(0x11111111), 0xCC693F55);
    }
}
