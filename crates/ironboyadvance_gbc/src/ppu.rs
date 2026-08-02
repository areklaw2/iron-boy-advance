use std::{cell::RefCell, rc::Rc};

use background::Background;
use getset::Getters;
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use ironboyadvance_sm83::GbMode;
use palette::{CgbPalette, Palette, color_index};
use registers::{LcdControl, LcdStatus, PpuMode};
use tile::{TILE_HEIGHT, TILE_WIDTH};
use window::Window;

use crate::{
    events::{FutureGbcEvent, GbcEvent, InterruptEvent, PpuEvent},
    ppu::{
        background::BgMapAttributes,
        oam::{OAM_SIZE, Oam},
    },
};

mod background;
mod oam;
mod palette;
pub mod registers;
mod tile;
mod window;

const FULL_WIDTH: usize = 256;
const NUMBER_OF_LINES: u8 = 154;
const VRAM_BANK_SIZE: usize = 0x2000;
const VRAM_SIZE: usize = 2 * VRAM_BANK_SIZE;
const TOTAL_LINE_CYCLES: usize = 456;
const OAM_SCAN_CYCLES: usize = 80;
const DRAWING_PIXELS_CYCLES: usize = 172;
const HBLANK_CYCLES: usize = 204;
const VBLANK_CYCLES: usize = TOTAL_LINE_CYCLES;

pub const CYCLES_PER_FRAME: usize = TOTAL_LINE_CYCLES * NUMBER_OF_LINES as usize;
pub const VIEWPORT_WIDTH: usize = 160;
pub const VIEWPORT_HEIGHT: usize = 144;

#[derive(Getters)]
pub struct Ppu {
    ly: u8,
    lyc: u8,
    lcd_control: LcdControl,
    lcd_status: LcdStatus,
    background: Background,
    window: Window,
    bg_palette: Palette,
    obj0_palette: Palette,
    obj1_palette: Palette,
    cgb_bg_palette: CgbPalette,
    cgb_obj_palette: CgbPalette,
    vram: Vec<u8>,
    oam: Oam,
    oam_buffer: Vec<(usize, u8)>,
    object_height: u8,
    line_priority: [(u8, bool); VIEWPORT_WIDTH],
    #[getset(get = "pub")]
    frame_buffer: Vec<u32>,
    vram_bank: usize,
    interrupt_line: bool,
    gb_mode: GbMode,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
    events: Vec<FutureGbcEvent>,
}

impl SystemMemoryAccess for Ppu {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[(self.vram_bank * VRAM_BANK_SIZE) | (address as usize & (VRAM_BANK_SIZE - 1))],
            0xFE00..=0xFE9F => self.oam.read_8(address),
            0xFF40 => self.lcd_control.into(),
            0xFF41 => self.lcd_status.into(),
            0xFF42 | 0xFF43 => self.background.read_8(address),
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => 0,
            0xFF47 => self.bg_palette.read(),
            0xFF48 => self.obj0_palette.read(),
            0xFF49 => self.obj1_palette.read(),
            0xFF4A | 0xFF4B => self.window.read_8(address),
            0xFF4C => 0xFF,
            0xFF4E => 0xFF,
            0xFF4F..=0xFF6B if self.gb_mode == GbMode::Monochrome => 0xFF,
            0xFF4F => self.vram_bank as u8 | 0xFE,
            0xFF68 => self.cgb_bg_palette.read_spec_and_index(),
            0xFF69 => self.cgb_bg_palette.read_palette(),
            0xFF6A => self.cgb_obj_palette.read_spec_and_index(),
            0xFF6B => self.cgb_obj_palette.read_palette(),
            _ => 0xFF,
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                self.vram[(self.vram_bank * VRAM_BANK_SIZE) | (address as usize & (VRAM_BANK_SIZE - 1))] = value
            }
            0xFE00..=0xFE9F => self.oam.write_8(address, value),
            0xFF40 => self.set_lcd_control(value),
            0xFF41 => self.write_lcd_status(value),
            0xFF42 | 0xFF43 => self.background.write_8(address, value),
            0xFF44 => {}
            0xFF45 => self.set_lyc(value),
            0xFF47 => self.bg_palette.write(value),
            0xFF48 => self.obj0_palette.write(value),
            0xFF49 => self.obj1_palette.write(value),
            0xFF4A | 0xFF4B => self.window.write_8(address, value),
            0xFF4C => {}
            0xFF4E => {}
            0xFF4F..=0xFF6B if self.gb_mode == GbMode::Monochrome => {}
            0xFF4F => self.vram_bank = (value & 0x01) as usize,
            0xFF68 => self.cgb_bg_palette.write_spec_and_index(value),
            0xFF69 => self.cgb_bg_palette.write_palette(value),
            0xFF6A => self.cgb_obj_palette.write_spec_and_index(value),
            0xFF6B => self.cgb_obj_palette.write_palette(value),
            _ => panic!("PPU does not handle write {:#04X}", address),
        }
    }
}

impl Ppu {
    pub fn new(mode: GbMode, skip_boot: bool, scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Ppu {
        let lcd_control = LcdControl::from_bits(match skip_boot {
            true => 0x91,
            false => 0x00,
        });

        let mut lcd_status = LcdStatus::new();
        if lcd_control.lcd_enabled() {
            lcd_status.set_mode(PpuMode::OamScan);
            scheduler
                .borrow_mut()
                .schedule((GbcEvent::Ppu(PpuEvent::OamScan), OAM_SCAN_CYCLES));
        }

        Ppu {
            ly: 0,
            lyc: 0,
            lcd_control,
            lcd_status,
            background: Background::new(),
            window: Window::new(),
            bg_palette: Palette::new(match skip_boot {
                true => 0xFC,
                false => 0x00,
            }),
            obj0_palette: Palette::new(0),
            obj1_palette: Palette::new(1),
            cgb_bg_palette: CgbPalette::new(),
            cgb_obj_palette: CgbPalette::new(),
            vram: vec![0; VRAM_SIZE],
            oam: Oam::new(),
            oam_buffer: Vec::new(),
            object_height: TILE_HEIGHT,
            line_priority: [(0, false); VIEWPORT_WIDTH],
            frame_buffer: vec![0xFFFFFF; VIEWPORT_WIDTH * VIEWPORT_HEIGHT],
            vram_bank: 0,
            interrupt_line: false,
            gb_mode: mode,
            scheduler,
            events: Vec::new(),
        }
    }

    pub fn handle_event(&mut self, ppu_event: PpuEvent, timestamp: usize) {
        match ppu_event {
            PpuEvent::OamScan => self.oam_scan_complete(),
            PpuEvent::DrawingPixels => self.drawing_pixels_complete(),
            PpuEvent::HBlank => self.h_blank_complete(),
            PpuEvent::VBlank => self.v_blank_complete(),
        }

        self.schedule_pending_events(timestamp);
    }

    fn oam_scan_complete(&mut self) {
        self.set_mode(PpuMode::DrawingPixels);
        self.events
            .push((GbcEvent::Ppu(PpuEvent::DrawingPixels), DRAWING_PIXELS_CYCLES));
    }

    fn drawing_pixels_complete(&mut self) {
        self.render_scanline();
        self.set_mode(PpuMode::HBlank);
        self.events.push((GbcEvent::Ppu(PpuEvent::HBlank), HBLANK_CYCLES));
    }

    fn h_blank_complete(&mut self) {
        self.window.increment_line_counter(self.lcd_control.window_enabled(), self.ly);

        match self.ly == VIEWPORT_HEIGHT as u8 - 1 {
            true => {
                self.events.push((GbcEvent::Interrupt(InterruptEvent::VBlank), 0));
                self.set_mode(PpuMode::VBlank);
                self.events.push((GbcEvent::Ppu(PpuEvent::VBlank), VBLANK_CYCLES));
            }
            false => {
                self.set_mode(PpuMode::OamScan);
                self.events.push((GbcEvent::Ppu(PpuEvent::OamScan), OAM_SCAN_CYCLES));
            }
        }

        self.set_ly(self.ly + 1);
    }

    fn v_blank_complete(&mut self) {
        self.set_ly(self.ly + 1);

        match self.ly {
            0 => {
                self.window.reset_line_counter();
                self.set_mode(PpuMode::OamScan);
                self.events.push((GbcEvent::Ppu(PpuEvent::OamScan), OAM_SCAN_CYCLES));
            }
            _ => self.events.push((GbcEvent::Ppu(PpuEvent::VBlank), VBLANK_CYCLES)),
        }
    }

    fn raise_lcd_interrupt(&mut self) {
        self.events.push((GbcEvent::Interrupt(InterruptEvent::Lcd), 0));
    }

    fn schedule_pending_events(&mut self, timestamp: usize) {
        for (event, delta) in self.events.drain(..) {
            self.scheduler.borrow_mut().schedule_at_timestamp(event, timestamp + delta);
        }
    }

    fn cancel_events(&mut self) {
        let mut scheduler = self.scheduler.borrow_mut();
        scheduler.cancel_events(GbcEvent::Ppu(PpuEvent::OamScan));
        scheduler.cancel_events(GbcEvent::Ppu(PpuEvent::DrawingPixels));
        scheduler.cancel_events(GbcEvent::Ppu(PpuEvent::HBlank));
        scheduler.cancel_events(GbcEvent::Ppu(PpuEvent::VBlank));
    }

    pub fn mode(&self) -> PpuMode {
        self.lcd_status.mode()
    }

    fn set_mode(&mut self, mode: PpuMode) {
        self.lcd_status.set_mode(mode);
        self.update_interrupt_line();
    }

    fn update_interrupt_line(&mut self) {
        let interrupt_line = (self.lcd_status.lyc_interrupt() && self.lcd_status.lyc_equals_ly())
            || match self.lcd_status.mode() {
                PpuMode::HBlank => self.lcd_status.mode0_interrupt(),
                PpuMode::VBlank => self.lcd_status.mode1_interrupt(),
                PpuMode::OamScan => self.lcd_status.mode2_interrupt(),
                PpuMode::DrawingPixels => false,
            };

        if interrupt_line && !self.interrupt_line {
            self.raise_lcd_interrupt();
        }

        self.interrupt_line = interrupt_line;
    }

    fn clear_screen(&mut self) {
        self.line_priority.fill((0, false));
        self.frame_buffer.fill(0xFFFFFF);
    }

    fn set_ly(&mut self, value: u8) {
        self.ly = value % NUMBER_OF_LINES;
        self.compare_line();
    }

    pub fn set_lyc(&mut self, value: u8) {
        self.lyc = value;
        self.compare_line();
        let timestamp = self.scheduler.borrow().timestamp();
        self.schedule_pending_events(timestamp);
    }

    fn compare_line(&mut self) {
        self.lcd_status.set_lyc_equals_ly(self.lyc == self.ly);
        self.update_interrupt_line();
    }

    fn write_lcd_status(&mut self, value: u8) {
        let status: u8 = self.lcd_status.into();
        self.lcd_status = (value & 0x78 | status & 0x07).into();
        self.update_interrupt_line();

        let timestamp = self.scheduler.borrow().timestamp();
        self.schedule_pending_events(timestamp);
    }

    fn set_lcd_control(&mut self, value: u8) {
        let was_enabled = self.lcd_control.lcd_enabled();
        self.lcd_control = value.into();

        match (was_enabled, self.lcd_control.lcd_enabled()) {
            (true, false) => self.disable_lcd(),
            (false, true) => self.enable_lcd(),
            _ => (),
        }
    }

    fn disable_lcd(&mut self) {
        self.cancel_events();
        self.clear_screen();
        self.window.reset_line_counter();
        self.set_ly(0);
        self.lcd_status.set_mode(PpuMode::HBlank);
        self.events.clear();
    }

    fn enable_lcd(&mut self) {
        self.set_ly(0);
        self.lcd_status.set_mode(PpuMode::OamScan);
        self.events.push((GbcEvent::Ppu(PpuEvent::OamScan), OAM_SCAN_CYCLES));
        let timestamp = self.scheduler.borrow().timestamp();
        self.schedule_pending_events(timestamp);
    }

    fn render_scanline(&mut self) {
        if self.lcd_control.bg_window_enabled() || self.gb_mode == GbMode::Color {
            self.render_bg_window_line();
        }

        if self.lcd_control.object_enabled() {
            self.render_object_line();
        }
    }

    fn render_bg_window_line(&mut self) {
        for lx in 0..VIEWPORT_WIDTH as u8 {
            let (tile_index_address, x_offset, y_offset) = self.bg_window_tile_data(lx);

            let tile_index = self.read_vram_bank_0(tile_index_address);
            let bg_map_attributes = if self.gb_mode == GbMode::Color {
                BgMapAttributes::from(self.read_vram_bank_1(tile_index_address))
            } else {
                BgMapAttributes::from(0)
            };

            let tile_address = self.lcd_control.tile_data_area().tile_address(tile_index);
            let (byte1, byte2) = match bg_map_attributes.y_flip() {
                false => self.get_tile_bytes(tile_address + y_offset as u16, bg_map_attributes.bank()),
                true => self.get_tile_bytes(tile_address + (14 - y_offset) as u16, bg_map_attributes.bank()),
            };

            let x_offset = match bg_map_attributes.x_flip() {
                false => x_offset,
                true => 7 - x_offset,
            };

            let color_index = color_index(byte1, byte2, x_offset);
            self.line_priority[lx as usize] = (color_index, bg_map_attributes.priority());

            let color = match self.gb_mode {
                GbMode::Color => self
                    .cgb_bg_palette
                    .pixel_color(bg_map_attributes.color_palette(), color_index),
                GbMode::ColorAsMonochrome => self.cgb_bg_palette.pixel_color(0, self.bg_palette.shade(color_index)),
                GbMode::Monochrome => self.bg_palette.pixel_color(color_index),
            };
            let offset = lx as usize + self.ly as usize * VIEWPORT_WIDTH;
            self.frame_buffer[offset] = color
        }
    }

    fn bg_window_tile_data(&self, lx: u8) -> (u16, u8, u8) {
        if self.window.inside_window(self.lcd_control.window_enabled(), lx, self.ly) {
            let (x, y) = self.window.tile_map_coordinates(lx);
            let tile_index_address = self.lcd_control.window_tile_map().tile_index_address(x, y);

            let (x_offset, y_offset) = self.window.pixel_offsets(lx, self.ly);
            (tile_index_address, x_offset, y_offset)
        } else {
            let (x, y) = self.background.tile_map_coordinates(lx, self.ly);
            let tile_index_address = self.lcd_control.bg_tile_map().tile_index_address(x, y);

            let (x_offset, y_offset) = self.background.pixel_offsets(x, y);
            (tile_index_address, x_offset, y_offset)
        }
    }

    fn render_object_line(&mut self) {
        self.read_objects_from_oam();
        for (oam_index, x_offset) in self.oam_buffer.iter() {
            let oam_entry = self.oam.oam_entry(*oam_index);
            let y_offset = oam_entry.y_position().wrapping_sub(16);

            let mut tile_index = oam_entry.tile_index();
            if self.object_height == 2 * TILE_HEIGHT {
                tile_index &= 0xFE;
            }

            let tile_base_address = 0x8000 + (tile_index as u16 * 16);
            let line_offset = if oam_entry.attributes().y_flip() {
                self.object_height - 1 - (self.ly - y_offset)
            } else {
                self.ly - y_offset
            };
            let tile_address = tile_base_address + line_offset as u16 * 2;

            let bank = oam_entry.attributes().bank();
            let (byte1, byte2) = self.get_tile_bytes(tile_address, bank);
            let color_palette_index = oam_entry.attributes().cgb_palette();

            for pixel_index in 0..TILE_WIDTH {
                let lx = x_offset.wrapping_add(pixel_index);
                if !(0..VIEWPORT_WIDTH).contains(&(lx as usize)) {
                    continue;
                }

                let oam_pixel_index = if oam_entry.attributes().x_flip() {
                    pixel_index
                } else {
                    7 - pixel_index
                };
                let color_index = color_index(byte1, byte2, oam_pixel_index);
                if color_index == 0 {
                    continue;
                }

                let offset = lx as usize + self.ly as usize * VIEWPORT_WIDTH;
                if self.gb_mode == GbMode::Color {
                    if self.line_priority[lx as usize].0 != 0
                        && self.lcd_control.bg_window_enabled()
                        && (self.line_priority[lx as usize].1 || oam_entry.attributes().priority())
                    {
                        continue;
                    }

                    let color = self.cgb_obj_palette.pixel_color(color_palette_index, color_index);
                    self.frame_buffer[offset] = color;
                } else {
                    if oam_entry.attributes().priority() && self.line_priority[lx as usize].0 != 0 {
                        continue;
                    }

                    let (object_palette, palette_number) = match oam_entry.attributes().dmg_palette() {
                        true => (self.obj1_palette, 1),
                        false => (self.obj0_palette, 0),
                    };

                    let color = match self.gb_mode {
                        GbMode::ColorAsMonochrome => self
                            .cgb_obj_palette
                            .pixel_color(palette_number, object_palette.shade(color_index)),
                        _ => object_palette.pixel_color(color_index),
                    };
                    self.frame_buffer[offset] = color;
                }
            }
        }
    }

    fn read_objects_from_oam(&mut self) {
        self.oam_buffer.clear();
        self.object_height = if self.lcd_control.object_size() {
            2 * TILE_HEIGHT
        } else {
            TILE_HEIGHT
        };

        for i in 0..OAM_SIZE {
            let oam_entry = self.oam.oam_entry(i);
            let object_y = oam_entry.y_position().wrapping_sub(16);
            let object_x = oam_entry.x_position().wrapping_sub(8);
            if self.ly >= object_y && self.ly < object_y.wrapping_add(self.object_height) {
                self.oam_buffer.push((i, object_x));
            }
        }

        if self.gb_mode == GbMode::Color {
            self.oam_buffer.sort_by_key(|entry| entry.0);
        } else {
            self.oam_buffer.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        }
        self.oam_buffer.truncate(10);
        self.oam_buffer.reverse();
    }

    fn get_tile_bytes(&self, address: u16, bank: bool) -> (u8, u8) {
        match bank {
            false => (self.read_vram_bank_0(address), self.read_vram_bank_0(address + 1)),
            true => (self.read_vram_bank_1(address), self.read_vram_bank_1(address + 1)),
        }
    }

    fn read_vram_bank_0(&self, address: u16) -> u8 {
        self.vram[address as usize - 0x8000]
    }

    fn read_vram_bank_1(&self, address: u16) -> u8 {
        self.vram[VRAM_BANK_SIZE + address as usize - 0x8000]
    }
}
