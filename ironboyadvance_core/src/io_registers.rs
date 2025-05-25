use ironboyadvance_arm7tdmi::memory::IoMemoryAccess;

pub struct IoRegisters {
    data: Vec<u8>,
}

impl IoRegisters {
    pub fn new() -> Self {
        IoRegisters { data: vec![0; 0x400] }
    }
}

impl IoMemoryAccess for IoRegisters {
    fn read_8(&self, address: u32) -> u8 {
        self.data[address as usize]
    }

    fn write_8(&mut self, address: u32, value: u8) {
        self.data[address as usize] = value
    }
}
