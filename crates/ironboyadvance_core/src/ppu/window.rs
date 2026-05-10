use bitfields::bitfield;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::{
    io_registers::RegisterOps,
    ppu::{ScanlineContext, VIEWPORT_WIDTH},
};

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct WindowDimension {
    end: u8,   // bits 0-7: rightmost/bottom-most + 1
    start: u8, // bits 8-15: leftmost/top-most
}

impl WindowDimension {
    pub fn contains(&self, coordinate: u8) -> bool {
        if self.end() < self.start() {
            coordinate < self.end() || coordinate >= self.start()
        } else {
            coordinate >= self.start() && coordinate < self.end()
        }
    }
}

impl RegisterOps<u16> for WindowDimension {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WindowControl {
    pub backgrounds: [bool; 4],
    pub object: bool,
    pub special_effect: bool,
}

impl WindowControl {
    pub fn no_windowing_control() -> Self {
        Self {
            backgrounds: [true; 4],
            object: true,
            special_effect: true,
        }
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
    window_0_special_effect: bool,
    #[bits(2)]
    _not_used_6_7: u8,
    window_1_bg0_enable: bool,
    window_1_bg1_enable: bool,
    window_1_bg2_enable: bool,
    window_1_bg3_enable: bool,
    window_1_obj_enable: bool,
    window_1_special_effect: bool,
    #[bits(2)]
    _not_used_14_15: u8,
}

impl WindowInside {
    pub fn window_0_control(&self) -> WindowControl {
        WindowControl {
            backgrounds: [
                self.window_0_bg0_enable(),
                self.window_0_bg1_enable(),
                self.window_0_bg2_enable(),
                self.window_0_bg3_enable(),
            ],
            object: self.window_0_obj_enable(),
            special_effect: self.window_0_special_effect(),
        }
    }

    pub fn window_1_control(&self) -> WindowControl {
        WindowControl {
            backgrounds: [
                self.window_1_bg0_enable(),
                self.window_1_bg1_enable(),
                self.window_1_bg2_enable(),
                self.window_1_bg3_enable(),
            ],
            object: self.window_1_obj_enable(),
            special_effect: self.window_1_special_effect(),
        }
    }
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
    outside_special_effect: bool,
    #[bits(2)]
    _not_used_6_7: u8,
    obj_window_bg0_enable: bool,
    obj_window_bg1_enable: bool,
    obj_window_bg2_enable: bool,
    obj_window_bg3_enable: bool,
    obj_window_obj_enable: bool,
    obj_window_special_effect: bool,
    #[bits(2)]
    _not_used_14_15: u8,
}

impl WindowOutside {
    pub fn window_outside_control(&self) -> WindowControl {
        WindowControl {
            backgrounds: [
                self.outside_bg0_enable(),
                self.outside_bg1_enable(),
                self.outside_bg2_enable(),
                self.outside_bg3_enable(),
            ],
            object: self.outside_obj_enable(),
            special_effect: self.outside_special_effect(),
        }
    }

    pub fn obj_window_control(&self) -> WindowControl {
        WindowControl {
            backgrounds: [
                self.obj_window_bg0_enable(),
                self.obj_window_bg1_enable(),
                self.obj_window_bg2_enable(),
                self.obj_window_bg3_enable(),
            ],
            object: self.obj_window_obj_enable(),
            special_effect: self.obj_window_special_effect(),
        }
    }
}

impl RegisterOps<u16> for WindowOutside {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

pub struct Window {
    win_x_dimensions: [WindowDimension; 2],
    win_y_dimensions: [WindowDimension; 2],
    win_inside: WindowInside,
    win_outside: WindowOutside,
}

impl Window {
    pub fn new() -> Self {
        Self {
            win_x_dimensions: [WindowDimension::from_bits(0); 2],
            win_y_dimensions: [WindowDimension::from_bits(0); 2],
            win_inside: WindowInside::from_bits(0),
            win_outside: WindowOutside::from_bits(0),
        }
    }

    pub fn build_win_control_line(
        &self,
        ctx: &ScanlineContext,
        win_obj_line: &[bool; VIEWPORT_WIDTH],
        win_control_line: &mut [WindowControl; VIEWPORT_WIDTH],
    ) {
        let inside_win0_y = self.win_y_dimensions[0].contains(ctx.v_count);
        let inside_win1_y = self.win_y_dimensions[1].contains(ctx.v_count);

        for x in 0..VIEWPORT_WIDTH {
            let inside_win0 = inside_win0_y && self.win_x_dimensions[0].contains(x as u8);
            let inside_win1 = inside_win1_y && self.win_x_dimensions[1].contains(x as u8);

            if ctx.lcd_control.window_0_display() && inside_win0 {
                win_control_line[x] = self.win_inside.window_0_control();
            } else if ctx.lcd_control.window_1_display() && inside_win1 {
                win_control_line[x] = self.win_inside.window_1_control();
            } else if ctx.lcd_control.obj_window_display() && win_obj_line[x] {
                win_control_line[x] = self.win_outside.obj_window_control();
            } else if ctx.lcd_control.any_window_enabled() {
                win_control_line[x] = self.win_outside.window_outside_control();
            }
        }
    }
}

impl SystemMemoryAccess for Window {
    fn read_8(&self, _address: u32) -> u8 {
        0
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // WIN0H, WIN1H, WIN0V, WIN1V
            0x04000040..=0x04000041 => self.win_x_dimensions[0].write_byte(address, value),
            0x04000042..=0x04000043 => self.win_x_dimensions[1].write_byte(address, value),
            0x04000044..=0x04000045 => self.win_y_dimensions[0].write_byte(address, value),
            0x04000046..=0x04000047 => self.win_y_dimensions[1].write_byte(address, value),
            // WININ, WINOUT
            0x04000048..=0x04000049 => self.win_inside.write_byte(address, value),
            0x0400004A..=0x0400004B => self.win_outside.write_byte(address, value),
            _ => panic!("Invalid byte write for Window register: {:#010X}", address),
        }
    }
}
