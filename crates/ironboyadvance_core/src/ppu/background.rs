use bitfields::bitfield;

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TextBgScreenEntry {
    #[bits(10)]
    tile_index: u16,
    horizontal_flip: bool,
    vertical_flip: bool,
    #[bits(4)]
    palette_bank: u8,
}

impl TextBgScreenEntry {
    pub fn apply_flip(self, tile_pixel_x: u8, tile_pixel_y: u8) -> (u8, u8) {
        let tile_pixel_x = if self.horizontal_flip() {
            7 - tile_pixel_x
        } else {
            tile_pixel_x
        };
        let tile_pixel_y = if self.vertical_flip() {
            7 - tile_pixel_y
        } else {
            tile_pixel_y
        };
        (tile_pixel_x, tile_pixel_y)
    }
}
#[bitfield(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RotationScalingBgScreenEntry {
    tile_index: u8,
}
