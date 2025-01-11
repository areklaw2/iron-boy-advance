use header::Header;

use crate::{memory::IoMemoryAccess, GbaError};

pub mod header;

const MAX_CARTRIDGE_BYTES: usize = 32 * 1024 * 1024;

pub struct Cartridge {
    header: Header,
    data: Vec<u8>,
}

impl Cartridge {
    pub fn load(buffer: Vec<u8>) -> Result<Cartridge, GbaError> {
        let header = Header::load(&buffer[0..228])?;
        println!("{}", header.game_title());
        println!("{}", header.game_code());

        let mut data = vec![0; MAX_CARTRIDGE_BYTES];
        data[..buffer.len()].clone_from_slice(&buffer);

        Ok(Cartridge { header, data })
    }
}
