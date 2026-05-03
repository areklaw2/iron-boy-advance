use bitfields::bitfield;

use crate::io_registers::RegisterOps;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FrameSelection {
    Frame0,
    Frame1,
}

impl FrameSelection {
    pub const fn from_bits(bits: u8) -> Self {
        use FrameSelection::*;
        match bits {
            0 => Frame0,
            1 => Frame1,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub fn base_address(self) -> usize {
        use FrameSelection::*;
        match self {
            Frame0 => 0,
            Frame1 => 0xA000,
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
