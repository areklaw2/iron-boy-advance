use header::Header;

pub mod header;

pub struct Cartridge {
    header: Header,
}

impl Cartridge {
    pub fn load(buffer: Vec<u8>) -> Cartridge {
        let header = Header::load(&buffer[0..192]);
        Cartridge { header }
    }
}
