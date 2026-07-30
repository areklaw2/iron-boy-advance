use getset::Getters;
use tracing::warn;

#[derive(Getters)]
#[allow(unused)]
pub struct Header {
    game_title: String,
    #[getset(get = "pub")]
    game_code: String,
}

impl Header {
    pub fn load(bytes: &[u8]) -> Header {
        let complement_check = bytes[0xBD];
        if complement_check != calculate_checksum(&bytes[0xA0..=0xBC]) {
            warn!("Cartridge checksum not valid")
        }

        Header {
            game_title: String::from_utf8_lossy(&bytes[0xA0..0xAC]).into_owned(),
            game_code: String::from_utf8_lossy(&bytes[0xAC..0xB0]).into_owned(),
        }
    }
}

fn calculate_checksum(bytes: &[u8]) -> u8 {
    let mut checksum = 0u8;
    for byte in bytes {
        checksum = checksum.wrapping_sub(*byte)
    }
    checksum = checksum.wrapping_sub(0x19);
    checksum
}
