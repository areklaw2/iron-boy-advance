#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorMode {
    Color16,  //4bpp
    Color256, //8bpp
}

impl ColorMode {
    pub const fn from_bits(bits: u8) -> Self {
        use ColorMode::*;
        match bits {
            0x0 => Color16,
            0x1 => Color256,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub fn bytes_per_tile(self) -> usize {
        use ColorMode::*;
        match self {
            Color16 => 32,
            Color256 => 64,
        }
    }

    pub fn palette_index(self, tile: &[u8], tile_pixel_x: u8, tile_pixel_y: u8, palette_bank: u8) -> u8 {
        use ColorMode::*;
        debug_assert!(palette_bank < 16, "palette_bank must fit in 4 bits");
        match self {
            Color16 => {
                let byte = tile[(tile_pixel_y * 4 + tile_pixel_x / 2) as usize];
                let nibble = if tile_pixel_x & 1 == 0 { byte & 0xF } else { byte >> 4 };
                if nibble == 0 { 0 } else { palette_bank * 16 + nibble }
            }
            Color256 => tile[(tile_pixel_y * 8 + tile_pixel_x) as usize],
        }
    }
}
