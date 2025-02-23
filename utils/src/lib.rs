use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

pub fn read_file(filename: &PathBuf) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut file = File::open(filename)?;
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
pub mod macros {
    #[macro_export]
    macro_rules! get_set {
        ($name:ident, $set_name:ident, $type:ty) => {
            pub fn $name(&self) -> $type {
                self.$name
            }

            pub fn $set_name(&mut self, value: $type) {
                self.$name = value
            }
        };
    }

    pub use get_set;
}
