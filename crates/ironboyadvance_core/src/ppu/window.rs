use bitfields::bitfield;

use crate::io_registers::RegisterOps;

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
