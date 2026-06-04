use bitfields::{bitfield, bitflag};

use ironboyadvance_common::register_ops::RegisterOps;

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum BgMode {
    Mode0 = 0x0,
    Mode1 = 0x1,
    Mode2 = 0x2,
    Mode3 = 0x3,
    Mode4 = 0x4,
    Mode5 = 0x5,
    #[base]
    Prohibited = 0xFF,
}

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum FrameSelection {
    #[base]
    Frame0 = 0,
    Frame1 = 1,
}

impl FrameSelection {
    pub fn base_address(self) -> usize {
        match self {
            Self::Frame0 => 0,
            Self::Frame1 => 0xA000,
        }
    }
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
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

impl LcdControl {
    pub fn bg_enabled(&self, bg: usize) -> bool {
        match bg {
            0 => self.screen_display_bg0(),
            1 => self.screen_display_bg1(),
            2 => self.screen_display_bg2(),
            3 => self.screen_display_bg3(),
            _ => unreachable!(),
        }
    }

    pub fn bg_mode_supported(&self, bg: usize) -> bool {
        let allowed_bg_mask = match self.bg_mode() {
            BgMode::Mode0 => 0b1111,
            BgMode::Mode1 => 0b0111,
            BgMode::Mode2 => 0b1100,
            BgMode::Mode3 | BgMode::Mode4 | BgMode::Mode5 => 0b0100,
            BgMode::Prohibited => 0,
        };

        (allowed_bg_mask >> bg) & 1 == 1
    }

    pub fn any_window_enabled(&self) -> bool {
        self.window_0_display() || self.window_1_display() || self.obj_window_display()
    }
}

impl RegisterOps<u16> for LcdControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct LcdStatus {
    v_blank_flag: bool,
    h_blank_flag: bool,
    v_counter_flag: bool,
    v_blank_irq_enabled: bool,
    h_blank_irq_enabled: bool,
    v_counter_irq_enabled: bool,
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
        self.write_bits(bits & 0xFF38);
    }
}
