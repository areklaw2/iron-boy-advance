use bitfields::bitfield;

use crate::io_registers::RegisterOps;

pub enum BgMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
    Prohibited,
}

impl BgMode {
    pub const fn from_bits(bits: u8) -> Self {
        use BgMode::*;
        match bits {
            0x0 => Mode0,
            0x1 => Mode1,
            0x2 => Mode2,
            0x3 => Mode3,
            0x4 => Mode4,
            0x5 => Mode5,
            _ => Prohibited,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct LcdControl {
    #[bits(3)]
    bg_mode: BgMode,
    cgb_mode: bool,
    display_frame_select: bool,
    h_blank_interval_free: bool,
    obj_character_vram_mapping: bool,
    forced_blank: bool,
    screen_display_bg0: bool,
    screen_display_bg1: bool,
    screen_display_bg2: bool,
    screen_display_bg3: bool,
    screen_display_obj: bool,
    window_0_display: bool,
    window_1_display: bool,
    obj_window_display: bool,
}

impl RegisterOps<u16> for LcdControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct LcdStatus {
    #[bits(1)]
    v_blank: bool,
    #[bits(1)]
    h_blank: bool,
    #[bits(1)]
    v_counter: bool,
    v_blank_irq_enable: bool,
    h_blank_irq_enable: bool,
    v_counter_irq_enable: bool,
    #[bits(2)]
    _reserved: u8,
    #[bits(8)]
    v_count_setting: u8,
}

impl RegisterOps<u16> for LcdStatus {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits & 0xFF38);
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BgControl {
    #[bits(2)]
    priority: u8,
    #[bits(2)]
    character_base_block: u8, // BG Tile Data
    #[bits(2)]
    _reserved: u8,
    mosiac: bool,
    colors: bool,
    #[bits(5)]
    screen_base_block: u8, // BG Map Data
    display_area_overflow: bool,
    #[bits(2)]
    screen_size: u8,
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
    not_used: u8,
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
    #[bits(1)]
    sign: bool,
    #[bits(4)]
    not_used: u8,
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
    #[bits(1)]
    sign: bool,
}

impl RegisterOps<u16> for BgAffineParameter {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct WindowDimension {
    end: u8,   // bits 0-7: rightmost/bottom-most + 1
    start: u8, // bits 8-15: leftmost/top-most
}

impl RegisterOps<u16> for WindowDimension {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct WindowInside {
    #[bits(1)]
    window_0_bg0_enable: bool,
    #[bits(1)]
    window_0_bg1_enable: bool,
    #[bits(1)]
    window_0_bg2_enable: bool,
    #[bits(1)]
    window_0_bg3_enable: bool,
    #[bits(1)]
    window_0_obj_enable: bool,
    #[bits(1)]
    window_0_color_special_effect: bool,
    #[bits(2)]
    not_used0: u8,
    #[bits(1)]
    window_1_bg0_enable: bool,
    #[bits(1)]
    window_1_bg1_enable: bool,
    #[bits(1)]
    window_1_bg2_enable: bool,
    #[bits(1)]
    window_1_bg3_enable: bool,
    #[bits(1)]
    window_1_obj_enable: bool,
    #[bits(1)]
    window_1_color_special_effect: bool,
    #[bits(2)]
    not_used1: u8,
}

impl RegisterOps<u16> for WindowInside {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct WindowOutside {
    #[bits(1)]
    outside_bg0_enable: bool,
    #[bits(1)]
    outside_bg1_enable: bool,
    #[bits(1)]
    outside_bg2_enable: bool,
    #[bits(1)]
    outside_bg3_enable: bool,
    #[bits(1)]
    outside_obj_enable: bool,
    #[bits(1)]
    outside_color_special_effect: bool,
    #[bits(2)]
    not_used0: u8,
    #[bits(1)]
    obj_window_bg0_enable: bool,
    #[bits(1)]
    obj_window_bg1_enable: bool,
    #[bits(1)]
    obj_window_bg2_enable: bool,
    #[bits(1)]
    obj_window_bg3_enable: bool,
    #[bits(1)]
    obj_window_obj_enable: bool,
    #[bits(1)]
    obj_window_color_special_effect: bool,
    #[bits(2)]
    not_used1: u8,
}

impl RegisterOps<u16> for WindowOutside {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MosiacSize {
    #[bits(4)]
    bg_mosaic_h_size: u8, // (minus 1)
    #[bits(4)]
    bg_mosaic_v_size: u8, // (minus 1)
    #[bits(4)]
    obj_mosaic_h_size: u8, // (minus 1)
    #[bits(4)]
    obj_mosaic_v_size: u8, // (minus 1)
    not_used: u16,
}

impl RegisterOps<u32> for MosiacSize {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.set_bits(bits);
    }
}

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
            0b01 => AlphaBlending,
            0b10 => BrightnessIncrease,
            0b11 => BrightnessDecrease,
            _ => None,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ColorSpecialEffectsSelection {
    #[bits(1)]
    bg0_1st_target_pixel: bool,
    #[bits(1)]
    bg1_1st_target_pixel: bool,
    #[bits(1)]
    bg2_1st_target_pixel: bool,
    #[bits(1)]
    bg3_1st_target_pixel: bool,
    #[bits(1)]
    obj_1st_target_pixel: bool,
    #[bits(1)]
    bd_1st_target_pixel: bool,
    #[bits(2)]
    color_special_effect: ColorSpecialEffect,
    #[bits(1)]
    bg0_2nd_target_pixel: bool,
    #[bits(1)]
    bg1_2nd_target_pixel: bool,
    #[bits(1)]
    bg2_2nd_target_pixel: bool,
    #[bits(1)]
    bg3_2nd_target_pixel: bool,
    #[bits(1)]
    obj_2nd_target_pixel: bool,
    #[bits(1)]
    bd_2nd_target_pixel: bool,
    #[bits(2)]
    not_used: u8,
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
    not_used1: u8,
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
    not_used: u32,
}

impl RegisterOps<u32> for BrightnessCoefficient {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.set_bits(bits);
    }
}
