use bitfields::bitfield;

use crate::ppu::registers::ColorMode;
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AffineMode {
    NoAffine,
    Affine,
    Hidden,
    AffineDouble,
}

impl AffineMode {
    pub const fn from_bits(bits: u8) -> Self {
        use AffineMode::*;
        match bits {
            0x0 => NoAffine,
            0x1 => Affine,
            0x2 => Hidden,
            0x3 => AffineDouble,
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
        use ObjectMode::*;
        match bits {
            0x0 => Normal,
            0x1 => SemiTransparent,
            0x2 => ObjectWindow,
            _ => Prohibited,
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
        use ObjectShape::*;
        match bits {
            0x0 => Square,
            0x1 => Horizontal,
            0x2 => Vertical,
            _ => Prohibited,
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
    mosiac: bool,
    #[bits(1)]
    color_mode: ColorMode,
    #[bits(2)]
    object_shape: ObjectShape,
}
