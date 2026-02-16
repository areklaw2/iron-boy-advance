use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

pub struct Memory {
    wram_board: Vec<u8>,
    wram_chip: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            wram_board: vec![0; 0x40000],
            wram_chip: vec![0; 0x8000],
        }
    }
}

impl SystemMemoryAccess for Memory {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            0x02000000..=0x02FFFFFF => self.wram_board[(address & 0x3FFFF) as usize],
            0x03000000..=0x03FFFFFF => self.wram_chip[(address & 0x7FFF) as usize],
            _ => panic!("Invalid byte read for Memory: {:08X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            0x02000000..=0x02FFFFFF => self.wram_board[(address & 0x3FFFF) as usize] = value,
            0x03000000..=0x03FFFFFF => self.wram_chip[(address & 0x7FFF) as usize] = value,
            _ => panic!("Invalid byte write for Memory: {:08X}", address),
        }
    }
}
