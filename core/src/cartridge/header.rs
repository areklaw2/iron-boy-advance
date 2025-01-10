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
    pub fn load(buffer: &[u8]) -> Header {
        Header {
            rom_entry_point: buffer[0x00..0x04].try_into().unwrap(),
            nintendo_logo: buffer[0x04..0xA0].try_into().unwrap(),
            game_title: String::from_utf8(buffer[0xA0..0xAC].to_vec()).unwrap(),
            game_code: String::from_utf8(buffer[0xAC..0xB0].to_vec()).unwrap(),
            maker_code: String::from_utf8(buffer[0xB0..0xB2].to_vec()).unwrap(),
            fixed_value: buffer[0xB2],
            main_unit_code: buffer[0xB3],
            device_type: buffer[0xB4],
            reserved_ares: buffer[0xB5..0xBC].try_into().unwrap(),
            software_version: buffer[0xBC],
            complement_check: buffer[0xBD],
            reserved_area: buffer[0xBE..0xC0].try_into().unwrap(),
            ram_entry_point: buffer[0xC0..0xC4].try_into().unwrap(),
            boot_mode: buffer[0xC4],
            id_number: buffer[0xC5],
            not_used: buffer[0xC6..0xE0].try_into().unwrap(),
            joybus_entry_point: buffer[0xE0..0xE4].try_into().unwrap(),
        }
    }

    pub fn game_title(&self) -> String {
        self.game_title.clone()
    }

    pub fn game_code(&self) -> String {
        self.game_code.clone()
    }
}
