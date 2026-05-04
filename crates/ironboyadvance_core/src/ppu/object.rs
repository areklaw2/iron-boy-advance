use bitfields::bitfield;
use getset::CopyGetters;

use crate::ppu::color::ColorMode;

const OBJECT_SIZES: [[(u16, u16); 4]; 3] = [
    [(8, 8), (16, 16), (32, 32), (64, 64)], // Square
    [(16, 8), (32, 8), (32, 16), (64, 32)], // Horizontal
    [(8, 16), (8, 32), (16, 32), (32, 64)], // Vertical
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AffineMode {
    NoAffine,
    Affine,
    Hidden,
    AffineDouble,
}

impl AffineMode {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::NoAffine,
            0x1 => Self::Affine,
            0x2 => Self::Hidden,
            0x3 => Self::AffineDouble,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectMode {
    Normal,
    SemiTransparent,
    ObjectWindow,
    Prohibited,
}

impl ObjectMode {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Normal,
            0x1 => Self::SemiTransparent,
            0x2 => Self::ObjectWindow,
            _ => Self::Prohibited,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectShape {
    Square,
    Horizontal,
    Vertical,
    Prohibited,
}

impl ObjectShape {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Square,
            0x1 => Self::Horizontal,
            0x2 => Self::Vertical,
            _ => Self::Prohibited,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectAttribute0 {
    y: u8,
    #[bits(2)]
    affine_mode: AffineMode,
    #[bits(2)]
    object_mode: ObjectMode,
    mosaic: bool,
    #[bits(1)]
    color_mode: ColorMode,
    #[bits(2)]
    object_shape: ObjectShape,
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectAttribute1Affine {
    #[bits(9)]
    x: u16,
    #[bits(5)]
    affine_index: u8,
    #[bits(2)]
    object_size: u8,
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectAttribute1Normal {
    #[bits(9)]
    x: u16,
    #[bits(3)]
    _not_used: u8,
    horizontal_flip: bool,
    vertical_flip: bool,
    #[bits(2)]
    object_size: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectAttribute1 {
    Affine(ObjectAttribute1Affine),
    Normal(ObjectAttribute1Normal),
}

impl ObjectAttribute1 {
    pub const fn from_bits(bits: u16, is_affine: bool) -> Self {
        match is_affine {
            true => Self::Affine(ObjectAttribute1Affine::from_bits(bits)),
            false => Self::Normal(ObjectAttribute1Normal::from_bits(bits)),
        }
    }

    pub const fn x(self) -> u16 {
        match self {
            Self::Affine(a) => a.x(),
            Self::Normal(n) => n.x(),
        }
    }

    pub const fn object_size(self) -> u8 {
        match self {
            Self::Affine(a) => a.object_size(),
            Self::Normal(n) => n.object_size(),
        }
    }

    pub const fn into_bits(self) -> u16 {
        match self {
            Self::Affine(a) => a.into_bits(),
            Self::Normal(n) => n.into_bits(),
        }
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectAttribute2 {
    #[bits(10)]
    tile_index: u16,
    #[bits(2)]
    priority: u8,
    #[bits(4)]
    palette_bank: u8,
}

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct ObjectEntry {
    attribute0: ObjectAttribute0,
    attribute1: ObjectAttribute1,
    attribute2: ObjectAttribute2,
}

impl ObjectEntry {
    pub fn from_oam(bytes: &[u8]) -> Self {
        let attribute0 = ObjectAttribute0::from_bits(u16::from_le_bytes([bytes[0], bytes[1]]));
        let is_affine = matches!(attribute0.affine_mode(), AffineMode::Affine | AffineMode::AffineDouble);
        Self {
            attribute0,
            attribute1: ObjectAttribute1::from_bits(u16::from_le_bytes([bytes[2], bytes[3]]), is_affine),
            attribute2: ObjectAttribute2::from_bits(u16::from_le_bytes([bytes[4], bytes[5]])),
        }
    }

    pub fn object_map_pixel_size(&self) -> Option<(u16, u16)> {
        let shape = self.attribute0.object_shape();
        if matches!(shape, ObjectShape::Prohibited) {
            return None;
        }
        let size = self.attribute1.object_size() as usize;
        Some(OBJECT_SIZES[shape as usize][size])
    }
}
