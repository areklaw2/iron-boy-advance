use bitfields::bitfield;

use crate::ppu::color::ColorMode;

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
    y_coordinate: u8,
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
    x_coordinate: u16,
    #[bits(5)]
    affine_index: u8,
    #[bits(2)]
    object_size: u8,
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectAttribute1Normal {
    #[bits(9)]
    x_coordinate: u16,
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
    pub const fn from_raw(raw: u16, is_affine: bool) -> Self {
        match is_affine {
            true => Self::Affine(ObjectAttribute1Affine::from_bits(raw)),
            false => Self::Normal(ObjectAttribute1Normal::from_bits(raw)),
        }
    }

    pub const fn x_coordinate(self) -> u16 {
        match self {
            Self::Affine(a) => a.x_coordinate(),
            Self::Normal(n) => n.x_coordinate(),
        }
    }

    pub const fn object_size(self) -> u8 {
        match self {
            Self::Affine(a) => a.object_size(),
            Self::Normal(n) => n.object_size(),
        }
    }
}
