use crate::memory::IoMemoryAccess;

pub struct Bios {
    data: Vec<u8>,
}

impl Bios {
    pub fn load(buffer: Vec<u8>) -> Bios {
        Bios { data: buffer }
    }
}

impl IoMemoryAccess for Bios {
    fn read_32(&self, address: u32) -> u32 {
        todo!()
    }

    fn read_16(&self, address: u32) -> u16 {
        todo!()
    }

    fn read_8(&self, address: u32) -> u8 {
        todo!()
    }

    fn write_8(&mut self, _address: u32, _value: u8) {
        panic!("Bios is read only")
    }
}
