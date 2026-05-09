use getset::Getters;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::{
    io_registers::RegisterOps,
    ppu::{
        background::{Background, allowed_backgrounds_by_mode},
        color::bgr555_to_rgb888,
        effects::*,
        lcd::*,
        object::{Object, ObjectPixel},
        window::*,
    },
    scheduler::event::{EventType, FutureEvent, InterruptEvent, PpuEvent},
};

const CYCLES_PER_PIXEL: usize = 4;

const HDRAW_PIXELS: usize = 240;
const HBLANK_PIXELS: usize = 68;
const HBLANK_FLAG_LAG: usize = 46;

pub const HDRAW_CYCLES: usize = HDRAW_PIXELS * CYCLES_PER_PIXEL + HBLANK_FLAG_LAG;
const HBLANK_CYCLES: usize = HBLANK_PIXELS * CYCLES_PER_PIXEL - HBLANK_FLAG_LAG;
const CYCLES_PER_SCANLINE: usize = HDRAW_CYCLES + HBLANK_CYCLES;

const VDRAW_SCANLINES: usize = 160;
const VBLANK_SCANLINES: usize = 68;
const VDRAW_CYCLES: usize = VDRAW_SCANLINES * CYCLES_PER_SCANLINE;
const VBLANK_CYCLES: usize = VBLANK_SCANLINES * CYCLES_PER_SCANLINE;

const MAX_V_COUNT: usize = VDRAW_SCANLINES + VBLANK_SCANLINES - 1;
const PIXEL_PER_FRAME: usize = HDRAW_PIXELS * VDRAW_SCANLINES;

pub const CYCLES_PER_FRAME: usize = VDRAW_CYCLES + VBLANK_CYCLES;
pub const VIEWPORT_WIDTH: usize = HDRAW_PIXELS;
pub const VIEWPORT_HEIGHT: usize = VDRAW_SCANLINES;

const SB_SIDE: u16 = 32;
const SB_ENTRIES: u16 = SB_SIDE * SB_SIDE;

const OBJ_VRAM_START: usize = 0x10000;

mod background;
mod bitmap;
mod color;
mod effects;
mod lcd;
mod object;
mod tiles;
mod window;

pub struct ScanlineContext<'a> {
    pub vram: &'a [u8],
    pub palette_ram: &'a [u8],
    pub lcd_control: &'a LcdControl,
    pub v_count: u8,
}

#[derive(Getters)]
pub struct Ppu {
    lcd_control: LcdControl,
    green_swap: bool,
    lcd_status: LcdStatus,
    v_count: u8,
    background: Background,
    object: Object,
    win_x_dimensions: [WindowDimension; 2],
    win_y_dimensions: [WindowDimension; 2],
    win_inside: WindowInside,
    win_outside: WindowOutside,
    mosiac_size: MosaicSize,
    color_special_effects_selection: ColorSpecialEffectsSelection,
    alpha_blending_coefficients: AlphaBlendingCoefficients,
    brightness_coefficient: BrightnessCoefficient,
    palette_ram: Vec<u8>,
    vram: Vec<u8>,
    #[getset(get = "pub")]
    frame_buffer: [u32; PIXEL_PER_FRAME],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            lcd_control: LcdControl::from_bits(0),
            green_swap: false,
            lcd_status: LcdStatus::from_bits(0),
            v_count: 0,
            background: Background::new(),
            object: Object::new(),
            win_x_dimensions: [WindowDimension::from_bits(0); 2],
            win_y_dimensions: [WindowDimension::from_bits(0); 2],
            win_inside: WindowInside::from_bits(0),
            win_outside: WindowOutside::from_bits(0),
            mosiac_size: MosaicSize::from_bits(0),
            color_special_effects_selection: ColorSpecialEffectsSelection::from_bits(0),
            alpha_blending_coefficients: AlphaBlendingCoefficients::from_bits(0),
            brightness_coefficient: BrightnessCoefficient::from_bits(0),
            palette_ram: vec![0; 0x400],
            vram: vec![0; 0x18000],
            frame_buffer: [0; PIXEL_PER_FRAME],
        }
    }
}

impl SystemMemoryAccess for Ppu {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // DISPCNT
            0x04000000..=0x04000001 => self.lcd_control.read_byte(address),
            // Green Swap
            0x04000002 => self.green_swap as u8,
            0x04000003 => 0,
            // DISPSTAT
            0x04000004..=0x04000005 => self.lcd_status.read_byte(address),
            // VCOUNT
            0x04000006..=0x04000007 => (self.v_count as u16).read_byte(address),
            // Background Registers
            0x04000008..=0x0400003F => self.background.read_8(address),
            // WIN0H, WIN1H, WIN0V, WIN1V, WININ, WINOUT, MOSIAC — write-only
            0x04000040..=0x0400004F => 0,
            // BLDCNT, BLDALPHA, BLDY
            0x04000050..=0x04000051 => self.color_special_effects_selection.read_byte(address),
            0x04000052..=0x04000053 => self.alpha_blending_coefficients.read_byte(address),
            0x04000054..=0x04000057 => self.brightness_coefficient.read_byte(address),
            // Palette RAM
            0x05000000..=0x05FFFFFF => self.palette_ram[(address & 0x3FF) as usize],
            // VRAM (with 128KB mirror)
            0x06000000..=0x06FFFFFF => {
                let offset = (address & 0x1FFFF) as usize;
                let index = if offset >= 0x18000 { offset - 0x8000 } else { offset };
                self.vram[index]
            }
            // OAM
            0x07000000..=0x07FFFFFF => self.object.read_8(address),
            _ => panic!("Invalid byte read for Ppu Register: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // DISPCNT
            0x04000000..=0x04000001 => self.lcd_control.write_byte(address, value),
            // Green Swap
            0x04000002 => self.green_swap = value & 0x1 != 0,
            0x04000003 => {}
            // DISPSTAT
            0x04000004..=0x04000005 => self.lcd_status.write_byte(address, value),
            // VCOUNT — read-only
            0x04000006..=0x04000007 => {}
            // Background Registers
            0x04000008..=0x0400003F => self.background.write_8(address, value),
            // WIN0H, WIN1H, WIN0V, WIN1V
            0x04000040..=0x04000041 => self.win_x_dimensions[0].write_byte(address, value),
            0x04000042..=0x04000043 => self.win_x_dimensions[1].write_byte(address, value),
            0x04000044..=0x04000045 => self.win_y_dimensions[0].write_byte(address, value),
            0x04000046..=0x04000047 => self.win_y_dimensions[1].write_byte(address, value),
            // WININ, WINOUT
            0x04000048..=0x04000049 => self.win_inside.write_byte(address, value),
            0x0400004A..=0x0400004B => self.win_outside.write_byte(address, value),
            // MOSIAC
            0x0400004C..=0x0400004F => self.mosiac_size.write_byte(address, value),
            // BLDCNT, BLDALPHA, BLDY
            0x04000050..=0x04000051 => self.color_special_effects_selection.write_byte(address, value),
            0x04000052..=0x04000053 => self.alpha_blending_coefficients.write_byte(address, value),
            0x04000054..=0x04000057 => self.brightness_coefficient.write_byte(address, value),
            // Palette RAM
            0x05000000..=0x05FFFFFF => self.palette_ram[(address & 0x3FF) as usize] = value,
            // VRAM (with 128KB mirror)
            0x06000000..=0x06FFFFFF => {
                let offset = (address & 0x1FFFF) as usize;
                let index = if offset >= 0x18000 { offset - 0x8000 } else { offset };
                self.vram[index] = value;
            }
            // OAM
            0x07000000..=0x07FFFFFF => self.object.write_8(address, value),
            _ => panic!("Invalid byte write for Ppu Register: {:#010X}", address),
        }
    }
}

impl Ppu {
    fn set_v_count(&mut self, value: u8) -> Option<InterruptEvent> {
        self.v_count = value;
        let is_match = self.lcd_status.v_count_setting() == self.v_count;
        self.lcd_status.set_v_counter_flag(is_match);
        match self.lcd_status.v_counter_irq_enable() && self.lcd_status.v_counter_flag() {
            true => Some(InterruptEvent::LcdVCounterMatch),
            false => None,
        }
    }

    pub fn handle_event(&mut self, event: PpuEvent) -> Vec<FutureEvent> {
        match event {
            PpuEvent::HDraw => self.handle_hdraw_complete(),
            PpuEvent::HBlank => self.handle_hblank_complete(),
            PpuEvent::VBlankHDraw => self.handle_vblank_hdraw_complete(),
            PpuEvent::VBlankHBlank => self.handle_vblank_hblank_complete(),
        }
    }

    fn handle_hdraw_complete(&mut self) -> Vec<FutureEvent> {
        let mut events = vec![];
        self.lcd_status.set_h_blank_flag(true);

        if self.lcd_status.h_blank_irq_enable() {
            events.push((EventType::Interrupt(InterruptEvent::LcdHBlank), 0));
        }

        self.background.advance_current_reference_points();
        events.push((EventType::Ppu(PpuEvent::HBlank), HBLANK_CYCLES));
        events
    }

    fn handle_hblank_complete(&mut self) -> Vec<FutureEvent> {
        let mut events = vec![];
        if let Some(v_count_match) = self.set_v_count(self.v_count + 1) {
            events.push((EventType::Interrupt(v_count_match), 0));
        }

        self.lcd_status.set_h_blank_flag(false);

        if (self.v_count as usize) < VDRAW_SCANLINES {
            self.render_scanline();
            events.push((EventType::Ppu(PpuEvent::HDraw), HDRAW_CYCLES));
        } else {
            self.lcd_status.set_v_blank_flag(true);

            if self.lcd_status.v_blank_irq_enable() {
                events.push((EventType::Interrupt(InterruptEvent::LcdVBlank), 0));
            }

            events.push((EventType::Ppu(PpuEvent::VBlankHDraw), HDRAW_CYCLES));
        }
        events
    }

    fn handle_vblank_hdraw_complete(&mut self) -> Vec<FutureEvent> {
        let mut events = vec![];
        self.lcd_status.set_h_blank_flag(true);

        if self.lcd_status.h_blank_irq_enable() {
            events.push((EventType::Interrupt(InterruptEvent::LcdHBlank), 0));
        }

        events.push((EventType::Ppu(PpuEvent::VBlankHBlank), HBLANK_CYCLES));
        events
    }

    fn handle_vblank_hblank_complete(&mut self) -> Vec<FutureEvent> {
        let mut events = vec![];
        self.lcd_status.set_h_blank_flag(false);

        if (self.v_count as usize) < MAX_V_COUNT {
            if let Some(v_count_match) = self.set_v_count(self.v_count + 1) {
                events.push((EventType::Interrupt(v_count_match), 0));
            }

            events.push((EventType::Ppu(PpuEvent::VBlankHDraw), HDRAW_CYCLES));
        } else {
            if let Some(v_count_match) = self.set_v_count(0) {
                events.push((EventType::Interrupt(v_count_match), 0));
            }

            self.lcd_status.set_v_blank_flag(false);
            self.background.reload_current_reference_points();
            self.render_scanline();
            events.push((EventType::Ppu(PpuEvent::HDraw), HDRAW_CYCLES));
        }
        events
    }

    fn render_scanline(&mut self) {
        if self.lcd_control.forced_blank() {
            let start = self.v_count as usize * HDRAW_PIXELS;
            self.frame_buffer[start..start + HDRAW_PIXELS].fill(bgr555_to_rgb888(0x7FFF));
            return;
        }

        let ctx = ScanlineContext {
            vram: &self.vram,
            palette_ram: &self.palette_ram,
            lcd_control: &self.lcd_control,
            v_count: self.v_count,
        };

        let mode = self.lcd_control.bg_mode();
        let allowed = allowed_backgrounds_by_mode(mode);
        let enabled = [
            self.lcd_control.screen_display_bg0(),
            self.lcd_control.screen_display_bg1(),
            self.lcd_control.screen_display_bg2(),
            self.lcd_control.screen_display_bg3(),
        ];

        let mut bg_order = [0usize; 4];
        let mut count = 0;
        for bg in 0..4 {
            if allowed[bg] && enabled[bg] {
                bg_order[count] = bg;
                count += 1
            }
        }
        bg_order[..count].sort_by_key(|&bg| self.background.priority(bg));

        let mut bg_lines = [[None; VIEWPORT_WIDTH]; 4];
        for &bg in &bg_order[..count] {
            match (mode, bg) {
                (BgMode::Mode0, _) | (BgMode::Mode1, 0 | 1) => {
                    tiles::render_text_scanline(&self.background, bg, &ctx, &mut bg_lines[bg])
                }
                (BgMode::Mode1, 2) | (BgMode::Mode2, 2 | 3) => {
                    tiles::render_affine_scanline(&self.background, bg, &ctx, &mut bg_lines[bg])
                }
                (BgMode::Mode3, 2) => bitmap::render_mode3(&ctx, &mut bg_lines[bg]),
                (BgMode::Mode4, 2) => bitmap::render_mode4(&ctx, &mut bg_lines[bg]),
                (BgMode::Mode5, 2) => bitmap::render_mode5(&ctx, &mut bg_lines[bg]),
                _ => {}
            }
        }

        let mut obj_line = [None; VIEWPORT_WIDTH];
        if self.lcd_control.screen_display_obj() {
            self.object.render_scanline(&ctx, &mut obj_line);
        }

        self.composite_scanline(&bg_order[..count], &bg_lines, &obj_line);
    }

    fn backdrop_color(&self) -> u16 {
        u16::from_le_bytes([self.palette_ram[0], self.palette_ram[1]])
    }

    fn composite_scanline(
        &mut self,
        bg_order: &[usize],
        bg_lines: &[[Option<u16>; VIEWPORT_WIDTH]; 4],
        obj_line: &[Option<ObjectPixel>; VIEWPORT_WIDTH],
    ) {
        let row = self.v_count as usize * VIEWPORT_WIDTH;
        let backdrop_color = self.backdrop_color();
        let bg_priorities = self.background.priorities();

        for (x, frame_pixel) in self.frame_buffer[row..row + VIEWPORT_WIDTH].iter_mut().enumerate() {
            let obj_pixel = obj_line[x];
            let obj_color = obj_pixel.map(|obj| obj.color);

            let color = bg_order
                .iter()
                .find_map(|&bg| match obj_pixel.is_some_and(|obj| obj.priority <= bg_priorities[bg]) {
                    true => obj_color,
                    false => bg_lines[bg][x],
                })
                .or(obj_color)
                .unwrap_or(backdrop_color);
            *frame_pixel = bgr555_to_rgb888(color);
        }
    }
}
