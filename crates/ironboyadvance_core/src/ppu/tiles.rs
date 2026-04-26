use bitfields::bitfield;

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TextBgScreenEntry {
    #[bits(10)]
    tile_index: u16,
    horizontal_flip: bool,
    vertical_flip: bool,
    #[bits(4)]
    palette_bank: u16,
}

#[bitfield(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RotationScalingBgScreenEntry {
    tile_index: u8,
}
