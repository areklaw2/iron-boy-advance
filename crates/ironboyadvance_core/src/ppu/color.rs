#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorMode {
    Color16,  //4bpp
    Color256, //8bpp
}

impl ColorMode {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Color16,
            0x1 => Self::Color256,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub fn bytes_per_tile(self) -> usize {
        match self {
            Self::Color16 => 32,
            Self::Color256 => 64,
        }
    }

    pub fn palette_index(self, tile: &[u8], tile_pixel_x: u8, tile_pixel_y: u8, palette_bank: u8) -> u8 {
        debug_assert!(palette_bank < 16, "palette_bank must fit in 4 bits");
        match self {
            Self::Color16 => {
                let byte = tile[(tile_pixel_y * 4 + tile_pixel_x / 2) as usize];
                let nibble = if tile_pixel_x & 1 == 0 { byte & 0xF } else { byte >> 4 };
                if nibble == 0 { 0 } else { palette_bank * 16 + nibble }
            }
            Self::Color256 => tile[(tile_pixel_y * 8 + tile_pixel_x) as usize],
        }
    }
}

pub fn bgr555_to_rgb888(color: u16) -> u32 {
    let r = (color & 0x1F) as u32;
    let g = ((color >> 5) & 0x1F) as u32;
    let b = ((color >> 10) & 0x1F) as u32;
    ((r << 3 | r >> 2) << 16) | ((g << 3 | g >> 2) << 8) | (b << 3 | b >> 2)
}
