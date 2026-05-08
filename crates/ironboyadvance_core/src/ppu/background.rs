use crate::{
    io_registers::RegisterOps,
    ppu::{SB_ENTRIES, SB_SIDE, color::ColorMode},
};
use bitfields::bitfield;
use ironboyadvance_arm7tdmi::bits::SignExtend;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScreenSize {
    Size0,
    Size1,
    Size2,
    Size3,
}

impl ScreenSize {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Size0,
            0x1 => Self::Size1,
            0x2 => Self::Size2,
            0x3 => Self::Size3,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub fn text_map_pixel_size(self) -> (u16, u16) {
        match self {
            Self::Size0 => (256, 256),
            Self::Size1 => (512, 256),
            Self::Size2 => (256, 512),
            Self::Size3 => (512, 512),
        }
    }

    pub fn affine_map_pixel_size(self) -> u16 {
        //affine maps are square
        match self {
            Self::Size0 => 128,
            Self::Size1 => 256,
            Self::Size2 => 512,
            Self::Size3 => 1024,
        }
    }

    pub fn text_screen_entry_index(self, map_tile_x: u16, map_tile_y: u16) -> u16 {
        let screen_block_columns = match self {
            Self::Size0 => 1,
            Self::Size1 => 2,
            Self::Size2 => 1,
            Self::Size3 => 2,
        };
        let screen_block_index = (map_tile_y / SB_SIDE) * screen_block_columns + (map_tile_x / SB_SIDE);
        screen_block_index * SB_ENTRIES + (map_tile_y % SB_SIDE) * SB_SIDE + (map_tile_x % SB_SIDE)
    }

    pub fn affine_screen_entry_index(self, map_tile_x: u16, map_tile_y: u16) -> u16 {
        map_tile_y * self.affine_map_pixel_size() / 8 + map_tile_x
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CharacterBaseBlock(u8);

impl CharacterBaseBlock {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & 0b11)
    }

    pub const fn into_bits(self) -> u8 {
        self.0
    }

    pub const fn vram_offset(self) -> usize {
        self.0 as usize * 0x4000
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScreenBaseBlock(u8);

impl ScreenBaseBlock {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & 0x1F)
    }

    pub const fn into_bits(self) -> u8 {
        self.0
    }

    pub const fn vram_offset(self) -> usize {
        self.0 as usize * 0x800
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DisplayAreaOverflow {
    Transparent,
    Wraparound,
}

impl DisplayAreaOverflow {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Transparent,
            0x1 => Self::Wraparound,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BgControl {
    #[bits(2)]
    priority: u8,
    #[bits(2)]
    character_base_block: CharacterBaseBlock, // BG Tile Data
    #[bits(2)]
    _reserved: u8,
    mosaic: bool,
    #[bits(1)]
    color_mode: ColorMode,
    #[bits(5)]
    screen_base_block: ScreenBaseBlock, // BG Map Data
    #[bits(1)]
    display_area_overflow: DisplayAreaOverflow, //Affine only
    #[bits(2)]
    screen_size: ScreenSize,
}

impl RegisterOps<u16> for BgControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BgOffset {
    #[bits(9)]
    offset: u16,
    #[bits(7)]
    _not_used_9_15: u8,
}

impl RegisterOps<u16> for BgOffset {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BgReferencePoint {
    fractional_portion: u8,
    #[bits(19)]
    interger_portion: u32,
    sign: bool,
    #[bits(4)]
    _not_used_28_31: u8,
}

impl BgReferencePoint {
    pub fn as_i32(self) -> i32 {
        self.into_bits().sign_extend(28)
    }
}

impl RegisterOps<u32> for BgReferencePoint {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.set_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BgAffineParameter {
    fractional_portion: u8,
    #[bits(7)]
    interger_portion: u8,
    sign: bool,
}

impl BgAffineParameter {
    pub fn as_i32(self) -> i32 {
        self.into_bits().sign_extend(16)
    }
}

impl RegisterOps<u16> for BgAffineParameter {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}
