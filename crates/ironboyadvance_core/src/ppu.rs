use getset::Getters;
use ironboyadvance_arm7tdmi::{bits::SignExtend, memory::SystemMemoryAccess};

use crate::{
    io_registers::RegisterOps,
    ppu::{
        background::*,
        effects::*,
        lcd::*,
        object::{AffineMode, ObjectEntry, ObjectPixel},
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

const BG_MODE_5_WIDTH: usize = 160;
const BG_MODE_5_HEIGHT: usize = 128;

const SB_SIDE: u16 = 32;
const SB_ENTRIES: u16 = SB_SIDE * SB_SIDE;

const OBJ_VRAM_START: usize = 0x10000;
const OBJ_2D_CHAR_MAP_TILES: u32 = 1024;

mod background;
mod color;
mod effects;
mod lcd;
mod object;
mod window;

#[derive(Getters)]
pub struct Ppu {
    lcd_control: LcdControl,
    green_swap: bool,
    lcd_status: LcdStatus,
    v_count: u8,
    bg_controls: [BgControl; 4],
    bg_x_offsets: [BgOffset; 4],
    bg_y_offsets: [BgOffset; 4],
    bg2_reference_points: [BgReferencePoint; 2],
    bg2_affine_parameters: [BgAffineParameter; 4],
    bg3_reference_points: [BgReferencePoint; 2],
    bg3_affine_parameters: [BgAffineParameter; 4],
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
    oam: Vec<u8>,
    obj_buffer: Vec<ObjectEntry>,
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
            bg_controls: [BgControl::from_bits(0); 4],
            bg_x_offsets: [BgOffset::from_bits(0); 4],
            bg_y_offsets: [BgOffset::from_bits(0); 4],
            bg2_reference_points: [BgReferencePoint::from_bits(0); 2],
            bg2_affine_parameters: [BgAffineParameter::from_bits(0); 4],
            bg3_reference_points: [BgReferencePoint::from_bits(0); 2],
            bg3_affine_parameters: [BgAffineParameter::from_bits(0); 4],
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
            oam: vec![0; 0x400],
            obj_buffer: Vec::with_capacity(128),
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
            // BG0CNT, BG1CNT, BG2CNT, BG3CNT
            0x04000008..=0x04000009 => self.bg_controls[0].read_byte(address),
            0x0400000A..=0x0400000B => self.bg_controls[1].read_byte(address),
            0x0400000C..=0x0400000D => self.bg_controls[2].read_byte(address),
            0x0400000E..=0x0400000F => self.bg_controls[3].read_byte(address),
            // BG0HOFS, BG0VOFS, BG1HOFS, BG1VOFS, BG2HOFS, BG2VOFS, BG3HOFS, BG3VOFS
            // BG2PA, BG2PB, BG2PC, BG2PD, BG2X_L, BG2X_H, BG2Y_L, BG2Y_H
            // BG3PA, BG3PB, BG3PC, BG3PD, BG3X_L, BG3X_H, BG3Y_L, BG3Y_H
            // WIN0H, WIN1H, WIN0V, WIN1V, WININ, WINOUT, MOSIAC
            0x04000010..=0x0400004F => 0,
            // BLDCNT, BLDALPHA, BLDY,
            0x04000050..=0x04000051 => self.color_special_effects_selection.read_byte(address),
            0x04000052..=0x04000053 => self.alpha_blending_coefficients.read_byte(address),
            0x04000054..=0x04000057 => self.brightness_coefficient.read_byte(address),
            // Access Memory
            0x05000000..=0x05FFFFFF => self.palette_ram[(address & 0x3FF) as usize],
            0x06000000..=0x06FFFFFF => {
                let offset = (address & 0x1FFFF) as usize; // 128KB mirror
                let index = if offset >= 0x18000 { offset - 0x8000 } else { offset };
                self.vram[index]
            }
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
            // VCOUNT
            0x04000006..=0x04000007 => {}
            // BG0CNT, BG1CNT, BG2CNT, BG3CNT
            0x04000008..=0x04000009 => self.bg_controls[0].write_byte(address, value),
            0x0400000A..=0x0400000B => self.bg_controls[1].write_byte(address, value),
            0x0400000C..=0x0400000D => self.bg_controls[2].write_byte(address, value),
            0x0400000E..=0x0400000F => self.bg_controls[3].write_byte(address, value),
            // BG0HOFS, BG0VOFS, BG1HOFS, BG1VOFS, BG2HOFS, BG2VOFS, BG3HOFS, BG3VOFS
            0x04000010..=0x04000011 => self.bg_x_offsets[0].write_byte(address, value),
            0x04000012..=0x04000013 => self.bg_y_offsets[0].write_byte(address, value),
            0x04000014..=0x04000015 => self.bg_x_offsets[1].write_byte(address, value),
            0x04000016..=0x04000017 => self.bg_y_offsets[1].write_byte(address, value),
            0x04000018..=0x04000019 => self.bg_x_offsets[2].write_byte(address, value),
            0x0400001A..=0x0400001B => self.bg_y_offsets[2].write_byte(address, value),
            0x0400001C..=0x0400001D => self.bg_x_offsets[3].write_byte(address, value),
            0x0400001E..=0x0400001F => self.bg_y_offsets[3].write_byte(address, value),
            // BG2PA, BG2PB, BG2PC, BG2PD
            0x04000020..=0x04000021 => self.bg2_affine_parameters[0].write_byte(address, value),
            0x04000022..=0x04000023 => self.bg2_affine_parameters[1].write_byte(address, value),
            0x04000024..=0x04000025 => self.bg2_affine_parameters[2].write_byte(address, value),
            0x04000026..=0x04000027 => self.bg2_affine_parameters[3].write_byte(address, value),
            // BG2X_L, BG2X_H, BG2Y_L, BG2Y_H
            0x04000028..=0x0400002B => self.bg2_reference_points[0].write_byte(address, value),
            0x0400002C..=0x0400002F => self.bg2_reference_points[1].write_byte(address, value),
            // BG3PA, BG3PB, BG3PC, BG3PD
            0x04000030..=0x04000031 => self.bg3_affine_parameters[0].write_byte(address, value),
            0x04000032..=0x04000033 => self.bg3_affine_parameters[1].write_byte(address, value),
            0x04000034..=0x04000035 => self.bg3_affine_parameters[2].write_byte(address, value),
            0x04000036..=0x04000037 => self.bg3_affine_parameters[3].write_byte(address, value),
            // BG3X_L, BG3X_H, BG3Y_L, BG3Y_H
            0x04000038..=0x0400003B => self.bg3_reference_points[0].write_byte(address, value),
            0x0400003C..=0x0400003F => self.bg3_reference_points[1].write_byte(address, value),
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
            // BLDCNT, BLDALPHA, BLDY,
            0x04000050..=0x04000051 => self.color_special_effects_selection.write_byte(address, value),
            0x04000052..=0x04000053 => self.alpha_blending_coefficients.write_byte(address, value),
            0x04000054..=0x04000057 => self.brightness_coefficient.write_byte(address, value),
            // Access Memory
            0x05000000..=0x05FFFFFF => self.palette_ram[(address & 0x3FF) as usize] = value,
            0x06000000..=0x06FFFFFF => {
                let offset = (address & 0x1FFFF) as usize; // 128KB mirror
                let index = if offset >= 0x18000 { offset - 0x8000 } else { offset };
                self.vram[index] = value;
            }
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
        bg_order[..count].sort_by_key(|&background| self.bg_controls[background].priority());

        let mut bg_lines = [[None; VIEWPORT_WIDTH]; 4]; //move this into the ppu at som epoint
        for &bg in &bg_order[..count] {
            match (mode, bg) {
                (BgMode::Mode0, _) | (BgMode::Mode1, 0 | 1) => self.render_text_bg_scanline(bg, &mut bg_lines[bg]),
                (BgMode::Mode1, 2) | (BgMode::Mode2, _) => self.render_affine_bg_scanline(bg, &mut bg_lines[bg]),
                (BgMode::Mode3, _) => self.render_mode3_scanline(&mut bg_lines[bg]),
                (BgMode::Mode4, _) => self.render_mode4_scanline(&mut bg_lines[bg]),
                (BgMode::Mode5, _) => self.render_mode5_scanline(&mut bg_lines[bg]),
                _ => {}
            }
        }

        let mut obj_line: [Option<ObjectPixel>; VIEWPORT_WIDTH] = [None; VIEWPORT_WIDTH];
        if self.lcd_control.screen_display_obj() {
            self.render_obj_scanline(&mut obj_line);
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
        let bg_priorities: [u8; 4] = [
            self.bg_controls[0].priority(),
            self.bg_controls[1].priority(),
            self.bg_controls[2].priority(),
            self.bg_controls[3].priority(),
        ];

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

    fn render_text_bg_scanline(&mut self, bg: usize, bg_line: &mut [Option<u16>; VIEWPORT_WIDTH]) {
        let y = self.v_count;
        let scroll_x = self.bg_x_offsets[bg].offset();
        let scroll_y = self.bg_y_offsets[bg].offset();
        let screen_size = self.bg_controls[bg].screen_size();
        let (map_width, map_height) = screen_size.text_map_pixel_size();
        let screen_block_base = self.bg_controls[bg].screen_base_block().vram_offset();
        let character_block_base = self.bg_controls[bg].character_base_block().vram_offset();
        let color_mode = self.bg_controls[bg].color_mode();
        let bytes_per_tile = color_mode.bytes_per_tile();

        for (x, pixel) in bg_line.iter_mut().enumerate() {
            let map_pixel_x = (x as u16 + scroll_x) % map_width;
            let map_pixel_y = (y as u16 + scroll_y) % map_height;
            let screen_entry_index = screen_size.text_screen_entry_index(map_pixel_x / 8, map_pixel_y / 8);

            let screen_entry_address = screen_block_base + screen_entry_index as usize * 2;
            let screen_entry = TextBgScreenEntry::from_bits(u16::from_le_bytes([
                self.vram[screen_entry_address],
                self.vram[screen_entry_address + 1],
            ]));

            let (tile_pixel_x, tile_pixel_y) = screen_entry.apply_flip((map_pixel_x % 8) as u8, (map_pixel_y % 8) as u8);
            let tile_address = character_block_base + screen_entry.tile_index() as usize * color_mode.bytes_per_tile();

            //if BG fetches into OBJ tiles VRAM (>= 0x10000) render transparent
            if tile_address + bytes_per_tile > OBJ_VRAM_START {
                continue;
            }

            let tile = &self.vram[tile_address..tile_address + color_mode.bytes_per_tile()];
            let palette_index = color_mode.palette_index(tile, tile_pixel_x, tile_pixel_y, screen_entry.palette_bank());
            if palette_index == 0 {
                continue;
            }

            let palette_address = palette_index as usize * 2;
            *pixel = Some(u16::from_le_bytes([
                self.palette_ram[palette_address],
                self.palette_ram[palette_address + 1],
            ]));
        }
    }

    fn render_affine_bg_scanline(&self, _bg: usize, _bg_line: &mut [Option<u16>; VIEWPORT_WIDTH]) {
        // TODO: affine
    }

    fn render_mode3_scanline(&mut self, bg_line: &mut [Option<u16>; VIEWPORT_WIDTH]) {
        let row = (self.v_count as usize) * VIEWPORT_WIDTH;
        for (x, pixel) in bg_line.iter_mut().enumerate() {
            let vram_address = (row + x) * 2;
            *pixel = Some(u16::from_le_bytes([self.vram[vram_address], self.vram[vram_address + 1]]));
        }
    }

    fn render_mode4_scanline(&mut self, bg_line: &mut [Option<u16>; VIEWPORT_WIDTH]) {
        let frame_base_address = self.lcd_control.display_frame_select().base_address();
        let row = (self.v_count as usize) * VIEWPORT_WIDTH;
        for (x, pixel) in bg_line.iter_mut().enumerate() {
            let palette_index = self.vram[frame_base_address + row + x];
            if palette_index == 0 {
                continue;
            }

            let palette_address = palette_index as usize * 2;
            *pixel = Some(u16::from_le_bytes([
                self.palette_ram[palette_address],
                self.palette_ram[palette_address + 1],
            ]));
        }
    }

    fn render_mode5_scanline(&mut self, bg_line: &mut [Option<u16>; VIEWPORT_WIDTH]) {
        let y = self.v_count as usize;
        if y >= BG_MODE_5_HEIGHT {
            return;
        }

        let frame_base_address = self.lcd_control.display_frame_select().base_address();
        for (x, pixel) in bg_line[..BG_MODE_5_WIDTH].iter_mut().enumerate() {
            let vram_address = frame_base_address + (y * BG_MODE_5_WIDTH + x) * 2;
            *pixel = Some(u16::from_le_bytes([self.vram[vram_address], self.vram[vram_address + 1]]));
        }
    }

    fn render_obj_scanline(&mut self, obj_line: &mut [Option<ObjectPixel>; VIEWPORT_WIDTH]) {
        let y = self.v_count;
        self.obj_buffer.clear();
        for obj_bytes in self.oam.chunks(8) {
            let obj_entry = ObjectEntry::from_oam(obj_bytes);
            if obj_entry.is_visible(y) {
                self.obj_buffer.push(obj_entry);
            }
        }

        for obj_entry in self.obj_buffer.iter().rev() {
            let attribute0 = obj_entry.attribute0();
            let attribute1 = obj_entry.attribute1();
            let attribute2 = obj_entry.attribute2();

            if attribute0.affine_mode() != AffineMode::NoAffine {
                //TODO implement affine
                continue;
            }

            let tile_index = attribute2.tile_index() as u32;
            if matches!(self.lcd_control.bg_mode(), BgMode::Mode3 | BgMode::Mode4 | BgMode::Mode5) && tile_index < 512 {
                continue;
            }

            let Some((obj_width, obj_height)) = obj_entry.obj_map_pixel_size() else {
                continue;
            };

            let Some((total_object_width, _)) = obj_entry.total_object_pixel_size() else {
                continue;
            };

            let color_mode = attribute0.color_mode();
            let bytes_per_tile = color_mode.bytes_per_tile() as u32;

            let obj_x = attribute1.x().sign_extend(9);
            let obj_y = attribute0.y();

            let obj_pixel_y = (y as u32).wrapping_sub(obj_y as u32) & 0xFF;
            let obj_pixel_y = attribute1.apply_v_flip(obj_pixel_y, obj_height);

            let obj_tile_y = obj_pixel_y / 8;
            let tile_pixel_y = (obj_pixel_y % 8) as u8;

            let tile_row_bytes: u32 = match self.lcd_control.obj_character_vram_mapping() {
                true => (obj_width as u32 / 8) * bytes_per_tile,
                false => OBJ_2D_CHAR_MAP_TILES,
            };

            let tile_row_base = OBJ_VRAM_START + (tile_index * 32).wrapping_add(obj_tile_y * tile_row_bytes) as usize;

            let start = (-obj_x).max(0);
            let end = (VIEWPORT_WIDTH as i32 - obj_x).min(total_object_width as i32);

            let palette_bank = attribute2.palette_bank();
            let object_mode = attribute0.object_mode();
            let priority = attribute2.priority();

            for obj_pixel_x in start..end {
                let screen_x = (obj_x + obj_pixel_x) as usize;
                let obj_pixel_x = attribute1.apply_h_flip(obj_pixel_x as u32, obj_width);

                let obj_tile_x = obj_pixel_x / 8;
                let tile_pixel_x = (obj_pixel_x % 8) as u8;

                let tile_address = tile_row_base + (obj_tile_x * bytes_per_tile) as usize;
                if tile_address + bytes_per_tile as usize > self.vram.len() {
                    continue;
                }

                let tile = &self.vram[tile_address..tile_address + bytes_per_tile as usize];
                let palette_index = color_mode.palette_index(tile, tile_pixel_x, tile_pixel_y, palette_bank);
                if palette_index == 0 {
                    continue;
                }

                let palette_address = 0x200 + palette_index as usize * 2;
                let color = u16::from_le_bytes([self.palette_ram[palette_address], self.palette_ram[palette_address + 1]]);
                obj_line[screen_x] = Some(ObjectPixel {
                    color,
                    priority,
                    object_mode,
                });
            }
        }
    }
}

fn allowed_backgrounds_by_mode(mode: BgMode) -> [bool; 4] {
    match mode {
        BgMode::Mode0 => [true, true, true, true],
        BgMode::Mode1 => [true, true, true, false],
        BgMode::Mode2 => [false, false, true, true],
        BgMode::Mode3 | BgMode::Mode4 | BgMode::Mode5 => [false, false, true, false],
        BgMode::Prohibited => [false; 4],
    }
}

fn bgr555_to_rgb888(color: u16) -> u32 {
    let r = (color & 0x1F) as u32;
    let g = ((color >> 5) & 0x1F) as u32;
    let b = ((color >> 10) & 0x1F) as u32;
    ((r << 3 | r >> 2) << 16) | ((g << 3 | g >> 2) << 8) | (b << 3 | b >> 2)
}
