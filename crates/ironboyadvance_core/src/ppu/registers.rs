use bitfields::bitfield;

use crate::io_registers::RegisterOps;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BgMode {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Prohibited,
}

impl BgMode {
    pub const fn from_bits(bits: u8) -> Self {
        use BgMode::*;
        match bits {
            0x0 => Zero,
            0x1 => One,
            0x2 => Two,
            0x3 => Three,
            0x4 => Four,
            0x5 => Five,
            _ => Prohibited,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FrameSelection {
    Zero,
    One,
}

impl FrameSelection {
    pub const fn from_bits(bits: u8) -> Self {
        use FrameSelection::*;
        match bits {
            0 => Zero,
            _ => One,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub fn base_address(self) -> usize {
        use FrameSelection::*;
        match self {
            Zero => 0,
            One => 0xA000,
        }
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct LcdControl {
    #[bits(3)]
    bg_mode: BgMode,
    cgb_mode: bool,
    #[bits(1)]
    display_frame_select: FrameSelection,
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
    v_blank_flag: bool,
    h_blank_flag: bool,
    v_counter_flag: bool,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScreenSize {
    Zero,
    One,
    Two,
    Three,
}

impl ScreenSize {
    pub const fn from_bits(bits: u8) -> Self {
        use ScreenSize::*;
        match bits {
            0x0 => Zero,
            0x1 => One,
            0x2 => Two,
            _ => Three,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub fn text_tile_map_size(self) -> (u16, u16) {
        use ScreenSize::*;
        match self {
            Zero => (32, 32),
            One => (64, 32),
            Two => (32, 64),
            Three => (64, 64),
        }
    }
    pub fn affine_tile_map_size(self) -> (u16, u16) {
        use ScreenSize::*;
        match self {
            Zero => (16, 16),
            One => (32, 32),
            Two => (64, 64),
            Three => (128, 128),
        }
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

    pub const fn vram_offset(self) -> u32 {
        self.0 as u32 * 0x4000
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

    pub const fn vram_offset(self) -> u32 {
        self.0 as u32 * 0x800
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
    colors: bool,
    #[bits(5)]
    screen_base_block: ScreenBaseBlock, // BG Map Data
    display_area_overflow: bool,
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
    not_used_9_15: u8,
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
    not_used_28_31: u8,
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
    window_0_bg0_enable: bool,
    window_0_bg1_enable: bool,
    window_0_bg2_enable: bool,
    window_0_bg3_enable: bool,
    window_0_obj_enable: bool,
    window_0_color_special_effect: bool,
    #[bits(2)]
    not_used_6_7: u8,
    window_1_bg0_enable: bool,
    window_1_bg1_enable: bool,
    window_1_bg2_enable: bool,
    window_1_bg3_enable: bool,
    window_1_obj_enable: bool,
    window_1_color_special_effect: bool,
    #[bits(2)]
    not_used_14_15: u8,
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
    outside_bg0_enable: bool,
    outside_bg1_enable: bool,
    outside_bg2_enable: bool,
    outside_bg3_enable: bool,
    outside_obj_enable: bool,
    outside_color_special_effect: bool,
    #[bits(2)]
    not_used_6_7: u8,
    obj_window_bg0_enable: bool,
    obj_window_bg1_enable: bool,
    obj_window_bg2_enable: bool,
    obj_window_bg3_enable: bool,
    obj_window_obj_enable: bool,
    obj_window_color_special_effect: bool,
    #[bits(2)]
    not_used_14_15: u8,
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
    not_used_16_31: u16,
}

impl RegisterOps<u32> for MosiacSize {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.set_bits(bits);
    }
}

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
