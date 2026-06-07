use std::{cell::RefCell, rc::Rc};

use getset::{CopyGetters, Getters};
use ironboyadvance_common::{memory::SystemMemoryAccess, register_ops::RegisterOps, scheduler::Scheduler};

use crate::{
    dma_control::RequestType,
    events::{DmaEvent, FutureGbaEvent, GbaEvent, InterruptEvent, PpuEvent},
    ppu::{
        background::Background, color::bgr555_to_rgb888, effects::Effects, lcd::*, mosaic::Mosaic, object::Object, window::*,
    },
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
const OBJ_PALETTE_START: usize = 0x200;

mod background;
mod bitmap;
mod color;
mod effects;
mod lcd;
mod mosaic;
mod object;
mod tiles;
mod window;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Layer {
    Bg(u8),
    Obj { semi_transparent: bool },
    Backdrop,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Pixel {
    pub color: u16,
    pub priority: u8,
    pub layer: Layer,
}

impl Pixel {
    pub fn backdrop(color: u16) -> Self {
        Self {
            color,
            priority: 4,
            layer: Layer::Backdrop,
        }
    }
}

pub struct ScanlineContext<'a> {
    pub vram: &'a [u8],
    pub palette_ram: &'a [u8],
    pub oam: &'a [u8],
    pub lcd_control: &'a LcdControl,
    pub mosaic: &'a Mosaic,
    pub v_count: u8,
}

#[derive(Getters, CopyGetters)]
pub struct Ppu {
    lcd_control: LcdControl,
    green_swap: bool,
    lcd_status: LcdStatus,
    v_count: u8,
    background: Background,
    object: Object,
    window: Window,
    mosaic: Mosaic,
    effects: Effects,
    palette_ram: Vec<u8>,
    vram: Vec<u8>,
    oam: Vec<u8>,
    #[getset(get = "pub")]
    frame_buffer: [u32; PIXEL_PER_FRAME],
    bg_lines: [[Option<Pixel>; VIEWPORT_WIDTH]; 4],
    obj_line: [Option<Pixel>; VIEWPORT_WIDTH],
    win_obj_line: [bool; VIEWPORT_WIDTH],
    win_control_line: [WindowControl; VIEWPORT_WIDTH],
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
}

impl Ppu {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbaEvent>>>) -> Self {
        scheduler
            .borrow_mut()
            .schedule((GbaEvent::Ppu(PpuEvent::HDraw), HDRAW_CYCLES));

        Self {
            lcd_control: LcdControl::from_bits(0),
            green_swap: false,
            lcd_status: LcdStatus::from_bits(0),
            v_count: 0,
            background: Background::new(),
            object: Object::new(),
            window: Window::new(),
            mosaic: Mosaic::new(),
            effects: Effects::new(),
            palette_ram: vec![0; 0x400],
            vram: vec![0; 0x18000],
            oam: vec![0; 0x400],
            frame_buffer: [0; PIXEL_PER_FRAME],
            bg_lines: [[None; VIEWPORT_WIDTH]; 4],
            obj_line: [None; VIEWPORT_WIDTH],
            win_obj_line: [false; VIEWPORT_WIDTH],
            win_control_line: [WindowControl::no_windowing_control(); VIEWPORT_WIDTH],
            scheduler,
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
            // WIN0H, WIN1H, WIN0V, WIN1V, WININ, WINOUT
            0x04000040..=0x0400004B => self.window.read_8(address),
            // MOSAIC
            0x0400004C..=0x0400004F => self.mosaic.read_8(address),
            // BLDCNT, BLDALPHA, BLDY
            0x04000050..=0x04000057 => self.effects.read_8(address),
            // Palette RAM
            0x05000000..=0x05FFFFFF => self.palette_ram[(address & 0x3FF) as usize],
            // VRAM (with 128KB mirror)
            0x06000000..=0x06FFFFFF => {
                let offset = (address & 0x1FFFF) as usize;
                let index = if offset >= 0x18000 { offset - 0x8000 } else { offset };
                self.vram[index]
            }
            // OAM
            0x07000000..=0x07FFFFFF => self.oam[(address & 0x3FF) as usize],
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
            // WIN0H, WIN1H, WIN0V, WIN1V, WININ, WINOUT
            0x04000040..=0x0400004B => self.window.write_8(address, value),
            // MOSAIC
            0x0400004C..=0x0400004F => self.mosaic.write_8(address, value),
            // BLDCNT, BLDALPHA, BLDY
            0x04000050..=0x04000057 => self.effects.write_8(address, value),
            // Palette RAM
            0x05000000..=0x05FFFFFF => self.palette_ram[(address & 0x3FF) as usize] = value,
            // VRAM (with 128KB mirror)
            0x06000000..=0x06FFFFFF => {
                let offset = (address & 0x1FFFF) as usize;
                let index = if offset >= 0x18000 { offset - 0x8000 } else { offset };
                self.vram[index] = value;
            }
            // OAM
            0x07000000..=0x07FFFFFF => self.oam[(address & 0x3FF) as usize] = value,
            _ => panic!("Invalid byte write for Ppu Register: {:#010X}", address),
        }
    }
}

impl Ppu {
    fn set_v_count(&mut self, value: u8) -> Option<InterruptEvent> {
        self.v_count = value;
        let is_match = self.lcd_status.v_count_setting() == self.v_count;
        self.lcd_status.set_v_counter_flag(is_match);
        match self.lcd_status.v_counter_irq_enabled() && self.lcd_status.v_counter_flag() {
            true => Some(InterruptEvent::LcdVCounterMatch),
            false => None,
        }
    }

    pub fn handle_event(&mut self, event: PpuEvent, timestamp: usize) {
        let events = match event {
            PpuEvent::HDraw => self.handle_hdraw_complete(),
            PpuEvent::HBlank => self.handle_hblank_complete(),
            PpuEvent::VBlankHDraw => self.handle_vblank_hdraw_complete(),
            PpuEvent::VBlankHBlank => self.handle_vblank_hblank_complete(),
        };

        for (event_type, delta) in events {
            self.scheduler
                .borrow_mut()
                .schedule_at_timestamp(event_type, timestamp + delta);
        }
    }

    fn handle_hdraw_complete(&mut self) -> Vec<FutureGbaEvent> {
        let mut events = vec![];
        self.render_scanline();
        self.lcd_status.set_h_blank_flag(true);

        if self.lcd_status.h_blank_irq_enabled() {
            events.push((GbaEvent::Interrupt(InterruptEvent::LcdHBlank), 0));
        }

        events.push((GbaEvent::Dma(DmaEvent::Request(RequestType::HBlank)), 0));
        if (2..=159).contains(&self.v_count) {
            events.push((GbaEvent::Dma(DmaEvent::Request(RequestType::Video)), 0));
        }

        self.advance_affine_points();
        events.push((GbaEvent::Ppu(PpuEvent::HBlank), HBLANK_CYCLES));
        events
    }

    fn handle_hblank_complete(&mut self) -> Vec<FutureGbaEvent> {
        let mut events = vec![];
        if let Some(v_count_match) = self.set_v_count(self.v_count + 1) {
            events.push((GbaEvent::Interrupt(v_count_match), 0));
        }

        self.lcd_status.set_h_blank_flag(false);

        if (self.v_count as usize) < VDRAW_SCANLINES {
            events.push((GbaEvent::Ppu(PpuEvent::HDraw), HDRAW_CYCLES));
        } else {
            self.lcd_status.set_v_blank_flag(true);

            if self.lcd_status.v_blank_irq_enabled() {
                events.push((GbaEvent::Interrupt(InterruptEvent::LcdVBlank), 0));
            }

            if self.v_count as usize == VIEWPORT_HEIGHT {
                events.push((GbaEvent::Dma(DmaEvent::Request(RequestType::VBlank)), 0));
            }

            events.push((GbaEvent::Ppu(PpuEvent::VBlankHDraw), HDRAW_CYCLES));
        }
        events
    }

    fn handle_vblank_hdraw_complete(&mut self) -> Vec<FutureGbaEvent> {
        let mut events = vec![];
        self.lcd_status.set_h_blank_flag(true);

        if self.lcd_status.h_blank_irq_enabled() {
            events.push((GbaEvent::Interrupt(InterruptEvent::LcdHBlank), 0));
        }

        if matches!(self.v_count, 160 | 161) {
            events.push((GbaEvent::Dma(DmaEvent::Request(RequestType::Video)), 0));
        }

        events.push((GbaEvent::Ppu(PpuEvent::VBlankHBlank), HBLANK_CYCLES));
        events
    }

    fn handle_vblank_hblank_complete(&mut self) -> Vec<FutureGbaEvent> {
        let mut events = vec![];
        self.lcd_status.set_h_blank_flag(false);

        if (self.v_count as usize) < MAX_V_COUNT {
            if let Some(v_count_match) = self.set_v_count(self.v_count + 1) {
                events.push((GbaEvent::Interrupt(v_count_match), 0));
            }

            if self.v_count == 162 {
                events.push((GbaEvent::Dma(DmaEvent::StopVideo), 0));
            }

            events.push((GbaEvent::Ppu(PpuEvent::VBlankHDraw), HDRAW_CYCLES));
        } else {
            if let Some(v_count_match) = self.set_v_count(0) {
                events.push((GbaEvent::Interrupt(v_count_match), 0));
            }

            self.lcd_status.set_v_blank_flag(false);
            self.background.reload_affine_points();
            events.push((GbaEvent::Ppu(PpuEvent::HDraw), HDRAW_CYCLES));
        }
        events
    }

    fn render_scanline(&mut self) {
        if self.lcd_control.forced_blank() {
            let start = self.v_count as usize * HDRAW_PIXELS;
            self.frame_buffer[start..start + HDRAW_PIXELS].fill(bgr555_to_rgb888(0x7FFF));
            return;
        }

        self.mosaic.update_sources(self.v_count);

        let ctx = ScanlineContext {
            vram: &self.vram,
            palette_ram: &self.palette_ram,
            oam: &self.oam,
            lcd_control: &self.lcd_control,
            mosaic: &self.mosaic,
            v_count: self.v_count,
        };

        for bg_line in &mut self.bg_lines {
            bg_line.fill(None);
        }
        self.obj_line.fill(None);
        self.win_obj_line.fill(false);
        self.win_control_line.fill(WindowControl::no_windowing_control());

        let mode = self.lcd_control.bg_mode();

        let mut bg_order = [0usize; 4];
        let mut count = 0;
        for bg in 0..4 {
            if self.lcd_control.bg_mode_supported(bg) && self.lcd_control.bg_enabled(bg) {
                bg_order[count] = bg;
                count += 1
            }
        }
        bg_order[..count].sort_by_key(|&bg| self.background.priority(bg));

        for &bg in &bg_order[..count] {
            match (mode, bg) {
                (BgMode::Mode0, _) | (BgMode::Mode1, 0 | 1) => {
                    self.background.render_text_scanline(&ctx, bg, &mut self.bg_lines[bg])
                }
                (BgMode::Mode1, 2) | (BgMode::Mode2, 2 | 3) => {
                    self.background.render_affine_scanline(&ctx, bg, &mut self.bg_lines[bg])
                }
                (BgMode::Mode3, 2) => self.background.render_mode3(&ctx, &mut self.bg_lines[bg]),
                (BgMode::Mode4, 2) => self.background.render_mode4(&ctx, &mut self.bg_lines[bg]),
                (BgMode::Mode5, 2) => self.background.render_mode5(&ctx, &mut self.bg_lines[bg]),
                _ => {}
            }
        }

        if self.lcd_control.screen_display_obj() {
            self.object
                .render_obj_scanline(&ctx, &mut self.obj_line, &mut self.win_obj_line);
        }

        self.window
            .build_win_control_line(&ctx, &self.win_obj_line, &mut self.win_control_line);

        self.composite_scanline(&bg_order[..count]);
    }

    fn composite_scanline(&mut self, bg_order: &[usize]) {
        let row = self.v_count as usize * VIEWPORT_WIDTH;
        let backdrop_pixel = Pixel::backdrop(u16::from_le_bytes([self.palette_ram[0], self.palette_ram[1]]));
        let bg_priorities = self.background.priorities();

        for (x, frame_pixel) in self.frame_buffer[row..row + VIEWPORT_WIDTH].iter_mut().enumerate() {
            let win_control = self.win_control_line[x];
            let mut obj_pixel = self.obj_line[x].filter(|_| win_control.object());

            let mut first_pixel: Option<Pixel> = None;
            let mut second_pixel: Option<Pixel> = None;

            for &bg in bg_order {
                if let Some(obj) = obj_pixel
                    && obj.priority <= bg_priorities[bg]
                {
                    obj_pixel = None;
                    if first_pixel.is_none() {
                        first_pixel = Some(obj);
                    } else {
                        second_pixel = Some(obj);
                        break;
                    }
                }

                if !win_control.background(bg) {
                    continue;
                }

                if let Some(pixel) = self.bg_lines[bg][x] {
                    if first_pixel.is_none() {
                        first_pixel = Some(pixel);
                    } else {
                        second_pixel = Some(pixel);
                        break;
                    }
                }
            }

            // OBJ at lower priority than every walked BG falls in here.
            if let Some(obj) = obj_pixel {
                if first_pixel.is_none() {
                    first_pixel = Some(obj);
                } else if second_pixel.is_none() {
                    second_pixel = Some(obj);
                }
            }

            let first = first_pixel.unwrap_or(backdrop_pixel);
            let second = second_pixel.unwrap_or(backdrop_pixel);
            let final_color = self.effects.resolve_pixel(first, second, win_control.special_effect());
            *frame_pixel = bgr555_to_rgb888(final_color);
        }
    }

    fn advance_affine_points(&mut self) {
        let next_y = self.v_count.wrapping_add(1);
        let bg_v_size = self.mosaic.size().bg_mosaic_v() + 1;
        let bg_mosaic_block_start = next_y.is_multiple_of(bg_v_size);
        let should_advance = [2, 3].map(|bg| !self.background.bg_control(bg).mosaic_enabled() || bg_mosaic_block_start);
        self.background.advance_affine_points(should_advance);
    }
}
