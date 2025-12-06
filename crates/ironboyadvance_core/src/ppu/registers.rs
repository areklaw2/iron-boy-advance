use bitfields::bitfield;

pub enum BgMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
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
            _ => unimplemented!(),
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
