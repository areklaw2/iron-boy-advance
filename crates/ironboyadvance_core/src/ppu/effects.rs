use bitfields::bitfield;

use crate::io_registers::RegisterOps;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorSpecialEffect {
    None,
    AlphaBlending,
    BrightnessIncrease,
    BrightnessDecrease,
}

impl ColorSpecialEffect {
    pub const fn from_bits(bits: u8) -> Self {
        use ColorSpecialEffect::*;
        match bits {
            0b00 => None,
            0b01 => AlphaBlending,
            0b10 => BrightnessIncrease,
            0b11 => BrightnessDecrease,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ColorSpecialEffectsSelection {
    bg0_1st_target_pixel: bool,
    bg1_1st_target_pixel: bool,
    bg2_1st_target_pixel: bool,
    bg3_1st_target_pixel: bool,
    obj_1st_target_pixel: bool,
    bd_1st_target_pixel: bool,
    #[bits(2)]
    color_special_effect: ColorSpecialEffect,
    bg0_2nd_target_pixel: bool,
    bg1_2nd_target_pixel: bool,
    bg2_2nd_target_pixel: bool,
    bg3_2nd_target_pixel: bool,
    obj_2nd_target_pixel: bool,
    bd_2nd_target_pixel: bool,
    #[bits(2)]
    not_used_14_15: u8,
}

impl RegisterOps<u16> for ColorSpecialEffectsSelection {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AlphaBlendingCoefficients {
    #[bits(5)]
    eva_coefficient: u8,
    #[bits(3)]
    not_used0: u8,
    #[bits(5)]
    evb_coefficient: u8,
    #[bits(3)]
    not_used_13_15: u8,
}

impl RegisterOps<u16> for AlphaBlendingCoefficients {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BrightnessCoefficient {
    #[bits(5)]
    evy_coefficient: u8,
    #[bits(27)]
    not_used_5_31: u32,
}

impl RegisterOps<u32> for BrightnessCoefficient {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.set_bits(bits);
    }
}

#[bitfield(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MosaicSize {
    #[bits(4)]
    bg_mosaic_h_size: u8, // (minus 1)
    #[bits(4)]
    bg_mosaic_v_size: u8, // (minus 1)
    #[bits(4)]
    obj_mosaic_h_size: u8, // (minus 1)
    #[bits(4)]
    obj_mosaic_v_size: u8, // (minus 1)
    not_used_16_31: u16,
}

impl RegisterOps<u32> for MosaicSize {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.set_bits(bits);
    }
}
