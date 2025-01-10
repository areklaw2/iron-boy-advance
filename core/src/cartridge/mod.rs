use header::Header;

pub mod header;

pub struct Cartridge {
    header: Header,
}

impl Cartridge {
    pub fn load(buffer: Vec<u8>) -> Cartridge {
        let header = Header::load(&buffer[0..228]);
        println!("{}", header.game_title());
        println!("{}", header.game_code());
        Cartridge { header }
    }
}
