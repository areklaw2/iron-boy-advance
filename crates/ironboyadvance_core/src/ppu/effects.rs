use bitfields::{bitfield, bitflag};
use ironboyadvance_common::{memory::SystemMemoryAccess, register_ops::RegisterOps};

use crate::ppu::{
    Layer, Pixel,
    color::{bgr555_to_channels, channels_to_bgr555},
};

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum SpecialEffect {
    #[base]
    None = 0b00,
    AlphaBlending = 0b01,
    BrightnessIncrease = 0b10,
    BrightnessDecrease = 0b11,
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct SpecialEffectsControl {
    bg0_1st_target_pixel: bool,
    bg1_1st_target_pixel: bool,
    bg2_1st_target_pixel: bool,
    bg3_1st_target_pixel: bool,
    obj_1st_target_pixel: bool,
    bd_1st_target_pixel: bool,
    #[bits(2)]
    color_special_effect: SpecialEffect,
    bg0_2nd_target_pixel: bool,
    bg1_2nd_target_pixel: bool,
    bg2_2nd_target_pixel: bool,
    bg3_2nd_target_pixel: bool,
    obj_2nd_target_pixel: bool,
    bd_2nd_target_pixel: bool,
    #[bits(2)]
    _not_used_14_15: u8,
}

impl RegisterOps<u16> for SpecialEffectsControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct AlphaBlending {
    #[bits(5)]
    eva_coefficient: u8,
    #[bits(3)]
    _not_used_0: u8,
    #[bits(5)]
    evb_coefficient: u8,
    #[bits(3)]
    _not_used_13_15: u8,
}

impl RegisterOps<u16> for AlphaBlending {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct Brightness {
    #[bits(5)]
    evy_coefficient: u8,
    #[bits(27)]
    _not_used_5_31: u32,
}

impl RegisterOps<u32> for Brightness {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.write_bits(bits);
    }
}

pub struct Effects {
    special_effect_control: SpecialEffectsControl,
    alpha_blending: AlphaBlending,
    brightness: Brightness,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            special_effect_control: SpecialEffectsControl::from_bits(0),
            alpha_blending: AlphaBlending::from_bits(0),
            brightness: Brightness::from_bits(0),
        }
    }

    pub fn effect(&self) -> SpecialEffect {
        self.special_effect_control.color_special_effect()
    }

    pub fn is_first_target(&self, layer: Layer) -> bool {
        match layer {
            Layer::Bg(0) => self.special_effect_control.bg0_1st_target_pixel(),
            Layer::Bg(1) => self.special_effect_control.bg1_1st_target_pixel(),
            Layer::Bg(2) => self.special_effect_control.bg2_1st_target_pixel(),
            Layer::Bg(3) => self.special_effect_control.bg3_1st_target_pixel(),
            Layer::Bg(_) => unreachable!(),
            Layer::Obj { .. } => self.special_effect_control.obj_1st_target_pixel(),
            Layer::Backdrop => self.special_effect_control.bd_1st_target_pixel(),
        }
    }

    pub fn is_second_target(&self, layer: Layer) -> bool {
        match layer {
            Layer::Bg(0) => self.special_effect_control.bg0_2nd_target_pixel(),
            Layer::Bg(1) => self.special_effect_control.bg1_2nd_target_pixel(),
            Layer::Bg(2) => self.special_effect_control.bg2_2nd_target_pixel(),
            Layer::Bg(3) => self.special_effect_control.bg3_2nd_target_pixel(),
            Layer::Bg(_) => unreachable!(),
            Layer::Obj { .. } => self.special_effect_control.obj_2nd_target_pixel(),
            Layer::Backdrop => self.special_effect_control.bd_2nd_target_pixel(),
        }
    }

    fn eva(&self) -> u8 {
        self.alpha_blending.eva_coefficient().min(16)
    }

    fn evb(&self) -> u8 {
        self.alpha_blending.evb_coefficient().min(16)
    }

    fn evy(&self) -> u8 {
        self.brightness.evy_coefficient().min(16)
    }

    pub fn resolve_pixel(&self, first_target: Pixel, second_target: Pixel, special_effect: bool) -> u16 {
        let translucent_object = matches!(first_target.layer, Layer::Obj { semi_transparent: true });

        if translucent_object && self.is_second_target(second_target.layer) {
            return alpha_blend(first_target.color, second_target.color, self.eva(), self.evb());
        }

        match special_effect {
            false => first_target.color,
            true => match self.effect() {
                SpecialEffect::None => first_target.color,
                SpecialEffect::AlphaBlending => {
                    if self.is_first_target(first_target.layer) && self.is_second_target(second_target.layer) {
                        alpha_blend(first_target.color, second_target.color, self.eva(), self.evb())
                    } else {
                        first_target.color
                    }
                }
                SpecialEffect::BrightnessIncrease => match self.is_first_target(first_target.layer) {
                    true => brighten(first_target.color, self.evy()),
                    false => first_target.color,
                },
                SpecialEffect::BrightnessDecrease => match self.is_first_target(first_target.layer) {
                    true => darken(first_target.color, self.evy()),
                    false => first_target.color,
                },
            },
        }
    }
}

impl SystemMemoryAccess for Effects {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // BLDCNT
            0x04000050..=0x04000051 => self.special_effect_control.read_byte(address),
            // BLDALPHA
            0x04000052..=0x04000053 => self.alpha_blending.read_byte(address),
            // BLDY — write-only
            0x04000054..=0x04000057 => 0,
            _ => panic!("Invalid byte read for Effects register: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // BLDCNT, BLDALPHA, BLDY
            0x04000050..=0x04000051 => self.special_effect_control.write_byte(address, value),
            0x04000052..=0x04000053 => self.alpha_blending.write_byte(address, value),
            0x04000054..=0x04000057 => self.brightness.write_byte(address, value),
            _ => panic!("Invalid byte write for Effects register: {:#010X}", address),
        }
    }
}

fn alpha_blend(first: u16, second: u16, eva: u8, evb: u8) -> u16 {
    let (first_red, first_green, first_blue) = bgr555_to_channels(first);
    let (second_red, second_green, second_blue) = bgr555_to_channels(second);
    let blend =
        |first: u16, second: u16| -> u16 { ((first as u32 * eva as u32 + second as u32 * evb as u32) / 16).min(31) as u16 };
    channels_to_bgr555(
        blend(first_red, second_red),
        blend(first_green, second_green),
        blend(first_blue, second_blue),
    )
}

fn brighten(color: u16, evy: u8) -> u16 {
    let (red, green, blue) = bgr555_to_channels(color);
    let increase = |color: u16| -> u16 { color + ((31 - color) as u32 * evy as u32 / 16) as u16 };
    channels_to_bgr555(increase(red), increase(green), increase(blue))
}

fn darken(color: u16, evy: u8) -> u16 {
    let (red, green, blue) = bgr555_to_channels(color);
    let decrease = |color: u16| -> u16 { color - (color as u32 * evy as u32 / 16) as u16 };
    channels_to_bgr555(decrease(red), decrease(green), decrease(blue))
}
