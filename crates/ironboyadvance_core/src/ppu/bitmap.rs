use crate::ppu::{Layer, Pixel, ScanlineContext, VIEWPORT_WIDTH, background::Background};

const BG_MODE_5_WIDTH: usize = 160;
const BG_MODE_5_HEIGHT: usize = 128;
const BITMAP_BG_INDEX: usize = 2;

impl Background {
    pub fn render_mode3(&self, ctx: &ScanlineContext, bg_line: &mut [Option<Pixel>; VIEWPORT_WIDTH]) {
        let priority = self.bg_control(BITMAP_BG_INDEX).priority();
        let layer = Layer::Bg(BITMAP_BG_INDEX as u8);
        let row = (ctx.v_count as usize) * VIEWPORT_WIDTH;
        for (x, pixel) in bg_line.iter_mut().enumerate() {
            let vram_address = (row + x) * 2;
            let color = u16::from_le_bytes([ctx.vram[vram_address], ctx.vram[vram_address + 1]]);
            *pixel = Some(Pixel { color, priority, layer });
        }
    }

    pub fn render_mode4(&self, ctx: &ScanlineContext, bg_line: &mut [Option<Pixel>; VIEWPORT_WIDTH]) {
        let priority = self.bg_control(BITMAP_BG_INDEX).priority();
        let layer = Layer::Bg(BITMAP_BG_INDEX as u8);
        let frame_base_address = ctx.lcd_control.display_frame_select().base_address();
        let row = (ctx.v_count as usize) * VIEWPORT_WIDTH;
        for (x, pixel) in bg_line.iter_mut().enumerate() {
            let palette_index = ctx.vram[frame_base_address + row + x];
            if palette_index == 0 {
                continue;
            }

            let palette_address = palette_index as usize * 2;
            let color = u16::from_le_bytes([ctx.palette_ram[palette_address], ctx.palette_ram[palette_address + 1]]);
            *pixel = Some(Pixel { color, priority, layer });
        }
    }

    pub fn render_mode5(&self, ctx: &ScanlineContext, bg_line: &mut [Option<Pixel>; VIEWPORT_WIDTH]) {
        let y = ctx.v_count as usize;
        if y >= BG_MODE_5_HEIGHT {
            return;
        }

        let priority = self.bg_control(BITMAP_BG_INDEX).priority();
        let layer = Layer::Bg(BITMAP_BG_INDEX as u8);
        let frame_base_address = ctx.lcd_control.display_frame_select().base_address();
        for (x, pixel) in bg_line[..BG_MODE_5_WIDTH].iter_mut().enumerate() {
            let vram_address = frame_base_address + (y * BG_MODE_5_WIDTH + x) * 2;
            let color = u16::from_le_bytes([ctx.vram[vram_address], ctx.vram[vram_address + 1]]);
            *pixel = Some(Pixel { color, priority, layer });
        }
    }
}
