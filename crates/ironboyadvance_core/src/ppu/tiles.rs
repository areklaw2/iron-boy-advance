use bitfields::bitfield;

use crate::ppu::{
    Layer, OBJ_VRAM_START, Pixel, ScanlineContext, VIEWPORT_WIDTH,
    background::{Background, DisplayAreaOverflow},
};

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TextBgScreenEntry {
    #[bits(10)]
    tile_index: u16,
    horizontal_flip: bool,
    vertical_flip: bool,
    #[bits(4)]
    palette_bank: u8,
}

impl TextBgScreenEntry {
    pub fn apply_flip(self, tile_pixel_x: u8, tile_pixel_y: u8) -> (u8, u8) {
        let tile_pixel_x = if self.horizontal_flip() {
            7 - tile_pixel_x
        } else {
            tile_pixel_x
        };
        let tile_pixel_y = if self.vertical_flip() {
            7 - tile_pixel_y
        } else {
            tile_pixel_y
        };
        (tile_pixel_x, tile_pixel_y)
    }
}

#[bitfield(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AffineBgScreenEntry {
    tile_index: u8,
}

impl Background {
    pub fn render_text_scanline(
        &self,
        ctx: &ScanlineContext,
        bg_index: usize,
        bg_line: &mut [Option<Pixel>; VIEWPORT_WIDTH],
    ) {
        let y = ctx.v_count;
        let scroll_x = self.bg_x_offset(bg_index).offset();
        let scroll_y = self.bg_y_offset(bg_index).offset();
        let bg_control = self.bg_control(bg_index);
        let screen_size = bg_control.screen_size();
        let (map_width, map_height) = screen_size.text_map_pixel_size();
        let screen_block_base = bg_control.screen_base_block().vram_offset();
        let character_block_base = bg_control.character_base_block().vram_offset();
        let color_mode = bg_control.color_mode();
        let bytes_per_tile = color_mode.bytes_per_tile();
        let priority = bg_control.priority();
        let layer = Layer::Bg(bg_index as u8);

        for (x, pixel) in bg_line.iter_mut().enumerate() {
            let map_pixel_x = (x as u16 + scroll_x) % map_width;
            let map_pixel_y = (y as u16 + scroll_y) % map_height;
            let screen_entry_index = screen_size.text_screen_entry_index(map_pixel_x / 8, map_pixel_y / 8);

            let screen_entry_address = screen_block_base + screen_entry_index as usize * 2;
            let screen_entry = TextBgScreenEntry::from_bits(u16::from_le_bytes([
                ctx.vram[screen_entry_address],
                ctx.vram[screen_entry_address + 1],
            ]));

            let (tile_pixel_x, tile_pixel_y) = screen_entry.apply_flip((map_pixel_x % 8) as u8, (map_pixel_y % 8) as u8);
            let tile_address = character_block_base + screen_entry.tile_index() as usize * bytes_per_tile;

            //if BG fetches into OBJ tiles VRAM (>= 0x10000) render transparent
            if tile_address + bytes_per_tile > OBJ_VRAM_START {
                continue;
            }

            let tile = &ctx.vram[tile_address..tile_address + bytes_per_tile];
            let palette_index = color_mode.palette_index(tile, tile_pixel_x, tile_pixel_y, screen_entry.palette_bank());
            if palette_index == 0 {
                continue;
            }

            let palette_address = palette_index as usize * 2;
            let color = u16::from_le_bytes([ctx.palette_ram[palette_address], ctx.palette_ram[palette_address + 1]]);
            *pixel = Some(Pixel { color, priority, layer });
        }
    }

    pub fn render_affine_scanline(
        &self,
        ctx: &ScanlineContext,
        bg_index: usize,
        bg_line: &mut [Option<Pixel>; VIEWPORT_WIDTH],
    ) {
        let pa = self.bg_pa(bg_index).as_i32();
        let pc = self.bg_pc(bg_index).as_i32();
        let bg_control = self.bg_control(bg_index);
        let screen_size = bg_control.screen_size();
        let map_size = screen_size.affine_map_pixel_size() as i32;
        let screen_block_base = bg_control.screen_base_block().vram_offset();
        let character_block_base = bg_control.character_base_block().vram_offset();
        let area_overflow = bg_control.display_area_overflow();
        let bytes_per_tile = 64;
        let bg_x_current = self.bg_x_current(bg_index);
        let bg_y_current = self.bg_y_current(bg_index);
        let priority = bg_control.priority();
        let layer = Layer::Bg(bg_index as u8);

        for (x, pixel) in bg_line.iter_mut().enumerate() {
            let mut map_pixel_x = (bg_x_current + pa * x as i32) >> 8;
            let mut map_pixel_y = (bg_y_current + pc * x as i32) >> 8;

            if !(0..map_size).contains(&map_pixel_x) || !(0..map_size).contains(&map_pixel_y) {
                match area_overflow {
                    DisplayAreaOverflow::Transparent => {
                        *pixel = None;
                        continue;
                    }
                    DisplayAreaOverflow::Wraparound => {
                        map_pixel_x = map_pixel_x.rem_euclid(map_size);
                        map_pixel_y = map_pixel_y.rem_euclid(map_size);
                    }
                }
            }

            let map_tile_x = (map_pixel_x as u16) / 8;
            let map_tile_y = (map_pixel_y as u16) / 8;
            let screen_entry_index = screen_size.affine_screen_entry_index(map_tile_x, map_tile_y);

            let screen_entry_address = screen_block_base + screen_entry_index as usize;
            let screen_entry = AffineBgScreenEntry::from_bits(ctx.vram[screen_entry_address]);

            let tile_pixel_x = (map_pixel_x % 8) as usize;
            let tile_pixel_y = (map_pixel_y % 8) as usize;
            let tile_address = character_block_base + screen_entry.tile_index() as usize * bytes_per_tile;

            //if BG fetches into OBJ tiles VRAM (>= 0x10000) render transparent
            if tile_address + bytes_per_tile > OBJ_VRAM_START {
                continue;
            }

            let palette_index = ctx.vram[tile_address + tile_pixel_y * 8 + tile_pixel_x];
            if palette_index == 0 {
                continue;
            }

            let palette_address = palette_index as usize * 2;
            let color = u16::from_le_bytes([ctx.palette_ram[palette_address], ctx.palette_ram[palette_address + 1]]);
            *pixel = Some(Pixel { color, priority, layer });
        }
    }
}
