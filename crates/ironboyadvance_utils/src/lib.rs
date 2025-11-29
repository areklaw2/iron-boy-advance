use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

pub mod bit;

pub fn read_file(filename: &PathBuf) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut file = File::open(filename)?;
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
