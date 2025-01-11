use std::str::from_utf8;

use crate::GbaError;

pub struct Header {
    rom_entry_point: [u8; 4],
    nintendo_logo: [u8; 156],
    game_title: String,
    game_code: String,
    maker_code: String,
    fixed_value: u8,
    main_unit_code: u8,
    device_type: u8,
    reserved_ares: [u8; 7],
    software_version: u8,
    complement_check: u8,
    reserved_area: [u8; 2],
    ram_entry_point: [u8; 4],
    boot_mode: u8,
    id_number: u8,
    not_used: [u8; 26],
    joybus_entry_point: [u8; 4],
}

impl Header {
    pub fn load(bytes: &[u8]) -> Result<Header, GbaError> {
        if bytes.len() < 0xE4 {
            return Err(GbaError::IncorrectHeaderLength);
        }

        let complement_check = bytes[0xBD];
        if complement_check != calculate_checksum(&bytes[0xA0..=0xBC]) {
            return Err(GbaError::CartridgeCheckSumFailure);
        } else {
            println!("Checksum passed!")
        }

        let game_title = from_utf8(&bytes[0xA0..0xAC]).map_err(|_| GbaError::HeaderParseFailure)?;
        let game_code = from_utf8(&bytes[0xAC..0xB0]).map_err(|_| GbaError::HeaderParseFailure)?;
        let maker_code = from_utf8(&bytes[0xB0..0xB2]).map_err(|_| GbaError::HeaderParseFailure)?;

        let header = Header {
            rom_entry_point: bytes[0x00..0x04].try_into().unwrap(),
            nintendo_logo: bytes[0x04..0xA0].try_into().unwrap(),
            game_title: String::from(game_title),
            game_code: String::from(game_code),
            maker_code: String::from(maker_code),
            fixed_value: bytes[0xB2],
            main_unit_code: bytes[0xB3],
            device_type: bytes[0xB4],
            reserved_ares: bytes[0xB5..0xBC].try_into().unwrap(),
            software_version: bytes[0xBC],
            complement_check,
            reserved_area: bytes[0xBE..0xC0].try_into().unwrap(),
            ram_entry_point: bytes[0xC0..0xC4].try_into().unwrap(),
            boot_mode: bytes[0xC4],
            id_number: bytes[0xC5],
            not_used: bytes[0xC6..0xE0].try_into().unwrap(),
            joybus_entry_point: bytes[0xE0..0xE4].try_into().unwrap(),
        };
        Ok(header)
    }

    pub fn game_title(&self) -> String {
        self.game_title.clone()
    }

    pub fn game_code(&self) -> String {
        self.game_code.clone()
    }
}

fn calculate_checksum(bytes: &[u8]) -> u8 {
    let mut checksum = 0u8;
    for i in 0..bytes.len() {
        checksum = checksum.wrapping_sub(bytes[i])
    }
    checksum = checksum.wrapping_sub(0x19);
    checksum
}
