use bitfields::bitfield;
use getset::CopyGetters;
use ironboyadvance_common::bits::SignExtend;

use crate::ppu::{
    Layer, OBJ_PALETTE_START, OBJ_VRAM_START, Pixel, ScanlineContext, VIEWPORT_WIDTH, color::ColorMode, lcd::BgMode,
};

const OBJ_2D_CHAR_MAP_TILES: u32 = 1024;

const OBJECT_SIZES: [[(u16, u16); 4]; 3] = [
    [(8, 8), (16, 16), (32, 32), (64, 64)], // Square
    [(16, 8), (32, 8), (32, 16), (64, 32)], // Horizontal
    [(8, 16), (8, 32), (16, 32), (32, 64)], // Vertical
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AffineMode {
    NoAffine,
    Affine,
    Hidden,
    AffineDouble,
}

impl AffineMode {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::NoAffine,
            0x1 => Self::Affine,
            0x2 => Self::Hidden,
            0x3 => Self::AffineDouble,
            _ => unreachable!(),
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectMode {
    Normal,
    SemiTransparent,
    ObjectWindow,
    Prohibited,
}

impl ObjectMode {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Normal,
            0x1 => Self::SemiTransparent,
            0x2 => Self::ObjectWindow,
            _ => Self::Prohibited,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectShape {
    Square,
    Horizontal,
    Vertical,
    Prohibited,
}

impl ObjectShape {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Square,
            0x1 => Self::Horizontal,
            0x2 => Self::Vertical,
            _ => Self::Prohibited,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectAttribute0 {
    y: u8,
    #[bits(2)]
    affine_mode: AffineMode,
    #[bits(2)]
    object_mode: ObjectMode,
    mosaic_enabled: bool,
    #[bits(1)]
    color_mode: ColorMode,
    #[bits(2)]
    object_shape: ObjectShape,
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectAttribute1Affine {
    #[bits(9)]
    x: u16,
    #[bits(5)]
    affine_index: u8,
    #[bits(2)]
    object_size: u8,
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectAttribute1Normal {
    #[bits(9)]
    x: u16,
    #[bits(3)]
    _not_used: u8,
    horizontal_flip: bool,
    vertical_flip: bool,
    #[bits(2)]
    object_size: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectAttribute1 {
    Affine(ObjectAttribute1Affine),
    Normal(ObjectAttribute1Normal),
}

impl ObjectAttribute1 {
    pub fn from_bits(bits: u16, is_affine: bool) -> Self {
        match is_affine {
            true => Self::Affine(ObjectAttribute1Affine::from_bits(bits)),
            false => Self::Normal(ObjectAttribute1Normal::from_bits(bits)),
        }
    }

    pub fn x(self) -> u16 {
        match self {
            Self::Affine(a) => a.x(),
            Self::Normal(n) => n.x(),
        }
    }

    pub fn object_size(self) -> u8 {
        match self {
            Self::Affine(a) => a.object_size(),
            Self::Normal(n) => n.object_size(),
        }
    }

    pub fn h_flip(self) -> bool {
        match self {
            Self::Normal(n) => n.horizontal_flip(),
            Self::Affine(_) => false,
        }
    }
    pub fn v_flip(self) -> bool {
        match self {
            Self::Normal(n) => n.vertical_flip(),
            Self::Affine(_) => false,
        }
    }
    pub fn affine_index(self) -> u8 {
        match self {
            Self::Affine(a) => a.affine_index(),
            Self::Normal(_) => 0,
        }
    }

    pub fn apply_h_flip(self, obj_pixel_x: u32, obj_width: u16) -> u32 {
        if self.h_flip() {
            obj_width as u32 - 1 - obj_pixel_x
        } else {
            obj_pixel_x
        }
    }

    pub fn apply_v_flip(self, obj_pixel_y: u32, obj_height: u16) -> u32 {
        if self.v_flip() {
            obj_height as u32 - 1 - obj_pixel_y
        } else {
            obj_pixel_y
        }
    }

    pub fn _into_bits(self) -> u16 {
        match self {
            Self::Affine(a) => a.into_bits(),
            Self::Normal(n) => n.into_bits(),
        }
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectAttribute2 {
    #[bits(10)]
    tile_index: u16,
    #[bits(2)]
    priority: u8,
    #[bits(4)]
    palette_bank: u8,
}

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct ObjectEntry {
    attribute0: ObjectAttribute0,
    attribute1: ObjectAttribute1,
    attribute2: ObjectAttribute2,
}

impl ObjectEntry {
    pub fn from_oam(bytes: &[u8]) -> Self {
        let attribute0 = ObjectAttribute0::from_bits(u16::from_le_bytes([bytes[0], bytes[1]]));
        let is_affine = matches!(attribute0.affine_mode(), AffineMode::Affine | AffineMode::AffineDouble);
        Self {
            attribute0,
            attribute1: ObjectAttribute1::from_bits(u16::from_le_bytes([bytes[2], bytes[3]]), is_affine),
            attribute2: ObjectAttribute2::from_bits(u16::from_le_bytes([bytes[4], bytes[5]])),
        }
    }

    pub fn obj_map_pixel_size(&self) -> Option<(u16, u16)> {
        let shape = self.attribute0.object_shape();
        if matches!(shape, ObjectShape::Prohibited) {
            return None;
        }
        let size = self.attribute1.object_size() as usize;
        Some(OBJECT_SIZES[shape as usize][size])
    }

    pub fn total_object_pixel_size(&self) -> Option<(u16, u16)> {
        let (width, height) = self.obj_map_pixel_size()?;
        match self.attribute0.affine_mode() {
            AffineMode::AffineDouble => Some((width * 2, height * 2)),
            _ => Some((width, height)),
        }
    }

    pub fn is_visible(&self, y: u8) -> bool {
        if self.attribute0.affine_mode() == AffineMode::Hidden {
            return false;
        }

        let Some((_, render_height)) = self.total_object_pixel_size() else {
            return false;
        };

        let object_row = (y as u32).wrapping_sub(self.attribute0.y() as u32) & 0xFF;
        object_row < render_height as u32
    }
}

pub struct Object {
    obj_buffer: Vec<ObjectEntry>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            obj_buffer: Vec::with_capacity(128),
        }
    }

    pub fn render_obj_scanline(
        &mut self,
        ctx: &ScanlineContext,
        obj_line: &mut [Option<Pixel>; VIEWPORT_WIDTH],
        win_obj_line: &mut [bool; VIEWPORT_WIDTH],
    ) {
        let y = ctx.v_count;
        self.obj_buffer.clear();
        for obj_bytes in ctx.oam.chunks(8) {
            let obj_entry = ObjectEntry::from_oam(obj_bytes);
            if obj_entry.is_visible(y) {
                self.obj_buffer.push(obj_entry);
            }
        }

        for obj_entry in self.obj_buffer.iter().rev() {
            let attribute0 = obj_entry.attribute0();
            let attribute1 = obj_entry.attribute1();
            let attribute2 = obj_entry.attribute2();

            let tile_index = attribute2.tile_index() as u32;
            if matches!(ctx.lcd_control.bg_mode(), BgMode::Mode3 | BgMode::Mode4 | BgMode::Mode5) && tile_index < 512 {
                continue;
            }

            let Some((obj_width, obj_height)) = obj_entry.obj_map_pixel_size() else {
                continue;
            };

            let Some((total_object_width, total_object_height)) = obj_entry.total_object_pixel_size() else {
                continue;
            };

            let color_mode = attribute0.color_mode();
            let bytes_per_tile = color_mode.bytes_per_tile() as u32;

            let obj_x = attribute1.x().sign_extend(9);
            let obj_y = attribute0.y();

            let tile_row_bytes: u32 = match ctx.lcd_control.obj_character_vram_mapping() {
                true => (obj_width as u32 / 8) * bytes_per_tile,
                false => OBJ_2D_CHAR_MAP_TILES,
            };

            let start = (-obj_x).max(0);
            let end = (VIEWPORT_WIDTH as i32 - obj_x).min(total_object_width as i32);

            let palette_bank = attribute2.palette_bank();
            let object_mode = attribute0.object_mode();
            let priority = attribute2.priority();
            let mosaic_enabled = attribute0.mosaic_enabled();
            let mosaic_h_block = ctx.mosaic.size().obj_mosaic_h() as i32 + 1;
            let y = match mosaic_enabled {
                true => ctx.mosaic.obj_source_y(),
                false => y,
            };

            match attribute0.affine_mode() {
                AffineMode::NoAffine => {
                    let obj_pixel_y = (y as u32).wrapping_sub(obj_y as u32) & 0xFF;
                    let obj_pixel_y = attribute1.apply_v_flip(obj_pixel_y, obj_height);

                    let obj_tile_y = obj_pixel_y / 8;
                    let tile_pixel_y = (obj_pixel_y % 8) as u8;

                    let tile_row_base = OBJ_VRAM_START + (tile_index * 32 + obj_tile_y * tile_row_bytes) as usize;

                    for obj_pixel_x in start..end {
                        let screen_x = (obj_x + obj_pixel_x) as usize;
                        let obj_pixel_x = match mosaic_enabled {
                            true => obj_pixel_x - obj_pixel_x.rem_euclid(mosaic_h_block),
                            false => obj_pixel_x,
                        };
                        let obj_pixel_x = attribute1.apply_h_flip(obj_pixel_x as u32, obj_width);

                        let obj_tile_x = obj_pixel_x / 8;
                        let tile_pixel_x = (obj_pixel_x % 8) as u8;

                        let tile_address = tile_row_base + (obj_tile_x * bytes_per_tile) as usize;
                        if tile_address + bytes_per_tile as usize > ctx.vram.len() {
                            continue;
                        }

                        let tile = &ctx.vram[tile_address..tile_address + bytes_per_tile as usize];
                        let palette_index = color_mode.palette_index(tile, tile_pixel_x, tile_pixel_y, palette_bank);
                        if palette_index == 0 {
                            continue;
                        }

                        match object_mode {
                            ObjectMode::ObjectWindow => win_obj_line[screen_x] = true,
                            _ => {
                                let palette_address = OBJ_PALETTE_START + palette_index as usize * 2;
                                let color = u16::from_le_bytes([
                                    ctx.palette_ram[palette_address],
                                    ctx.palette_ram[palette_address + 1],
                                ]);
                                obj_line[screen_x] = Some(Pixel {
                                    color,
                                    priority,
                                    layer: Layer::Obj {
                                        semi_transparent: matches!(object_mode, ObjectMode::SemiTransparent),
                                    },
                                });
                            }
                        }
                    }
                }
                AffineMode::Affine | AffineMode::AffineDouble => {
                    let (pa, pb, pc, pd) = read_obj_affine_matrix(ctx.oam, attribute1.affine_index());
                    let bounding_box_pixel_y = ((y as u32).wrapping_sub(obj_y as u32) & 0xFF) as i32;
                    let screen_offset_y = bounding_box_pixel_y - total_object_height as i32 / 2;

                    for bounding_box_pixel_x in start..end {
                        let screen_x = (obj_x + bounding_box_pixel_x) as usize;
                        let bounding_box_pixel_x = match mosaic_enabled {
                            true => bounding_box_pixel_x - bounding_box_pixel_x.rem_euclid(mosaic_h_block),
                            false => bounding_box_pixel_x,
                        };
                        let screen_offset_x = bounding_box_pixel_x - total_object_width as i32 / 2;

                        let obj_pixel_x = obj_width as i32 / 2 + ((pa * screen_offset_x + pb * screen_offset_y) >> 8);
                        let obj_pixel_y = obj_height as i32 / 2 + ((pc * screen_offset_x + pd * screen_offset_y) >> 8);

                        if !(0..obj_width as i32).contains(&obj_pixel_x) || !(0..obj_height as i32).contains(&obj_pixel_y) {
                            continue;
                        }

                        let obj_tile_x = obj_pixel_x as u32 / 8;
                        let obj_tile_y = obj_pixel_y as u32 / 8;
                        let tile_pixel_x = (obj_pixel_x % 8) as u8;
                        let tile_pixel_y = (obj_pixel_y % 8) as u8;

                        let tile_row_base = OBJ_VRAM_START + (tile_index * 32 + obj_tile_y * tile_row_bytes) as usize;
                        let tile_address = tile_row_base + (obj_tile_x * bytes_per_tile) as usize;

                        if tile_address + bytes_per_tile as usize > ctx.vram.len() {
                            continue;
                        }

                        let tile = &ctx.vram[tile_address..tile_address + bytes_per_tile as usize];
                        let palette_index = color_mode.palette_index(tile, tile_pixel_x, tile_pixel_y, palette_bank);
                        if palette_index == 0 {
                            continue;
                        }

                        match object_mode {
                            ObjectMode::ObjectWindow => win_obj_line[screen_x] = true,
                            _ => {
                                let palette_address = OBJ_PALETTE_START + palette_index as usize * 2;
                                let color = u16::from_le_bytes([
                                    ctx.palette_ram[palette_address],
                                    ctx.palette_ram[palette_address + 1],
                                ]);
                                obj_line[screen_x] = Some(Pixel {
                                    color,
                                    priority,
                                    layer: Layer::Obj {
                                        semi_transparent: matches!(object_mode, ObjectMode::SemiTransparent),
                                    },
                                });
                            }
                        }
                    }
                }
                AffineMode::Hidden => {}
            }
        }
    }
}

fn read_obj_affine_matrix(oam: &[u8], obj_index: u8) -> (i32, i32, i32, i32) {
    let base_address = (obj_index as usize) * 32;
    let pa = i16::from_le_bytes([oam[base_address + 0x06], oam[base_address + 0x07]]) as i32;
    let pb = i16::from_le_bytes([oam[base_address + 0x0E], oam[base_address + 0x0F]]) as i32;
    let pc = i16::from_le_bytes([oam[base_address + 0x16], oam[base_address + 0x17]]) as i32;
    let pd = i16::from_le_bytes([oam[base_address + 0x1E], oam[base_address + 0x1F]]) as i32;
    (pa, pb, pc, pd)
}
