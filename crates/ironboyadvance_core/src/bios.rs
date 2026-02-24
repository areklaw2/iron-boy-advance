use getset::CopyGetters;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

#[derive(Debug, CopyGetters)]
pub struct Bios {
    data: Box<[u8]>,
    #[getset(get_copy = "pub")]
    loaded: bool,
}

impl Bios {
    pub fn load(buffer: Box<[u8]>) -> Bios {
        let (data, loaded) = match !buffer.is_empty() {
            true => (buffer, true),
            false => (Box::new([0u8; 0x4000]) as Box<[u8]>, false),
        };
        Self { data, loaded }
    }
}

impl SystemMemoryAccess for Bios {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            0..=0x3FFF => self.data[address as usize],
            _ => panic!("Invalid byte read for Bios: {:08X}", address),
        }
    }

    fn write_8(&mut self, _address: u32, _value: u8) {}
}
