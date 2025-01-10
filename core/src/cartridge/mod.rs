use header::Header;

use crate::GbaError;

pub mod header;

pub struct Cartridge {
    header: Header,
}

impl Cartridge {
    pub fn load(buffer: Vec<u8>) -> Result<Cartridge, GbaError> {
        let header = Header::load(&buffer[0..228])?;
        println!("{}", header.game_title());
        println!("{}", header.game_code());
        Ok(Cartridge { header })
    }
}
