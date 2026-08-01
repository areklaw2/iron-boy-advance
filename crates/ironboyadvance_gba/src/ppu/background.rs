use bitfields::{bitfield, bitflag};
use ironboyadvance_common::{bits::SignExtend, memory::SystemMemoryAccess, register_ops::RegisterOps};

use crate::ppu::{SB_ENTRIES, SB_SIDE, color::ColorMode};

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum ScreenSize {
    #[base]
    Size0 = 0x0,
    Size1 = 0x1,
    Size2 = 0x2,
    Size3 = 0x3,
}

impl ScreenSize {
    pub fn text_map_pixel_size(self) -> (u16, u16) {
        match self {
            Self::Size0 => (256, 256),
            Self::Size1 => (512, 256),
            Self::Size2 => (256, 512),
            Self::Size3 => (512, 512),
        }
    }

    pub fn affine_map_pixel_size(self) -> u16 {
        //affine maps are square
        match self {
            Self::Size0 => 128,
            Self::Size1 => 256,
            Self::Size2 => 512,
            Self::Size3 => 1024,
        }
    }

    pub fn text_screen_entry_index(self, map_tile_x: u16, map_tile_y: u16) -> u16 {
        let screen_block_columns = match self {
            Self::Size0 => 1,
            Self::Size1 => 2,
            Self::Size2 => 1,
            Self::Size3 => 2,
        };
        let screen_block_index = (map_tile_y / SB_SIDE) * screen_block_columns + (map_tile_x / SB_SIDE);
        screen_block_index * SB_ENTRIES + (map_tile_y % SB_SIDE) * SB_SIDE + (map_tile_x % SB_SIDE)
    }

    pub fn affine_screen_entry_index(self, map_tile_x: u16, map_tile_y: u16) -> u16 {
        map_tile_y * (self.affine_map_pixel_size() / 8) + map_tile_x
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CharacterBaseBlock(u8);

impl CharacterBaseBlock {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & 0b11)
    }

    pub const fn into_bits(self) -> u8 {
        self.0
    }

    pub const fn vram_offset(self) -> usize {
        self.0 as usize * 0x4000
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScreenBaseBlock(u8);

impl ScreenBaseBlock {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & 0x1F)
    }

    pub const fn into_bits(self) -> u8 {
        self.0
    }

    pub const fn vram_offset(self) -> usize {
        self.0 as usize * 0x800
    }
}

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum DisplayAreaOverflow {
    #[base]
    Transparent = 0x0,
    Wraparound = 0x1,
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct BgControl {
    #[bits(2)]
    priority: u8,
    #[bits(2)]
    character_base_block: CharacterBaseBlock, // BG Tile Data
    #[bits(2)]
    _reserved: u8,
    mosaic_enabled: bool,
    #[bits(1)]
    color_mode: ColorMode,
    #[bits(5)]
    screen_base_block: ScreenBaseBlock, // BG Map Data
    #[bits(1)]
    display_area_overflow: DisplayAreaOverflow, //Affine only
    #[bits(2)]
    screen_size: ScreenSize,
}

impl RegisterOps<u16> for BgControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct BgOffset {
    #[bits(9)]
    offset: u16,
    #[bits(7)]
    _not_used_9_15: u8,
}

impl RegisterOps<u16> for BgOffset {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct BgReferencePoint {
    fractional_portion: u8,
    #[bits(19)]
    interger_portion: u32,
    sign: bool,
    #[bits(4)]
    _not_used_28_31: u8,
}

impl BgReferencePoint {
    pub fn as_i32(self) -> i32 {
        self.into_bits().sign_extend(28)
    }
}

impl RegisterOps<u32> for BgReferencePoint {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.write_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct BgAffineParameter {
    fractional_portion: u8,
    #[bits(7)]
    interger_portion: u8,
    sign: bool,
}

impl BgAffineParameter {
    pub fn as_i32(self) -> i32 {
        self.into_bits().sign_extend(16)
    }
}

impl RegisterOps<u16> for BgAffineParameter {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }
}

pub struct Background {
    bg_controls: [BgControl; 4],
    bg_x_offsets: [BgOffset; 4],
    bg_y_offsets: [BgOffset; 4],
    bg_x_references: [BgReferencePoint; 2],
    bg_y_references: [BgReferencePoint; 2],
    bg_x_affine: [i32; 2],
    bg_y_affine: [i32; 2],
    bg_pa: [BgAffineParameter; 2],
    bg_pb: [BgAffineParameter; 2],
    bg_pc: [BgAffineParameter; 2],
    bg_pd: [BgAffineParameter; 2],
}

impl Background {
    pub fn new() -> Self {
        Self {
            bg_controls: [BgControl::from_bits(0); 4],
            bg_x_offsets: [BgOffset::from_bits(0); 4],
            bg_y_offsets: [BgOffset::from_bits(0); 4],
            bg_x_references: [BgReferencePoint::from_bits(0); 2],
            bg_y_references: [BgReferencePoint::from_bits(0); 2],
            bg_x_affine: [0; 2],
            bg_y_affine: [0; 2],
            bg_pa: [BgAffineParameter::from_bits(0); 2],
            bg_pb: [BgAffineParameter::from_bits(0); 2],
            bg_pc: [BgAffineParameter::from_bits(0); 2],
            bg_pd: [BgAffineParameter::from_bits(0); 2],
        }
    }

    pub fn priority(&self, bg: usize) -> u8 {
        self.bg_controls[bg].priority()
    }

    pub fn priorities(&self) -> [u8; 4] {
        [
            self.bg_controls[0].priority(),
            self.bg_controls[1].priority(),
            self.bg_controls[2].priority(),
            self.bg_controls[3].priority(),
        ]
    }

    pub fn bg_control(&self, bg_index: usize) -> BgControl {
        self.bg_controls[bg_index]
    }

    pub fn bg_x_offset(&self, bg_index: usize) -> BgOffset {
        self.bg_x_offsets[bg_index]
    }

    pub fn bg_y_offset(&self, bg_index: usize) -> BgOffset {
        self.bg_y_offsets[bg_index]
    }

    pub fn bg_pa(&self, bg_index: usize) -> BgAffineParameter {
        self.bg_pa[bg_index - 2]
    }

    pub fn bg_pc(&self, bg_index: usize) -> BgAffineParameter {
        self.bg_pc[bg_index - 2]
    }

    pub fn bg_x_affine(&self, bg_index: usize) -> i32 {
        self.bg_x_affine[bg_index - 2]
    }

    pub fn bg_y_affine(&self, bg_index: usize) -> i32 {
        self.bg_y_affine[bg_index - 2]
    }

    pub fn advance_affine_points(&mut self, should_advance: [bool; 2]) {
        if should_advance[0] {
            self.bg_x_affine[0] = self.bg_x_affine[0].wrapping_add(self.bg_pb[0].as_i32());
            self.bg_y_affine[0] = self.bg_y_affine[0].wrapping_add(self.bg_pd[0].as_i32());
        }
        if should_advance[1] {
            self.bg_x_affine[1] = self.bg_x_affine[1].wrapping_add(self.bg_pb[1].as_i32());
            self.bg_y_affine[1] = self.bg_y_affine[1].wrapping_add(self.bg_pd[1].as_i32());
        }
    }

    pub fn reload_affine_points(&mut self) {
        self.bg_x_affine[0] = self.bg_x_references[0].as_i32();
        self.bg_y_affine[0] = self.bg_y_references[0].as_i32();
        self.bg_x_affine[1] = self.bg_x_references[1].as_i32();
        self.bg_y_affine[1] = self.bg_y_references[1].as_i32();
    }
}

impl SystemMemoryAccess for Background {
    type Address = u32;

    fn read_8(&self, address: u32) -> u8 {
        match address {
            // BG0CNT, BG1CNT, BG2CNT, BG3CNT
            0x04000008..=0x04000009 => self.bg_controls[0].read_byte(address),
            0x0400000A..=0x0400000B => self.bg_controls[1].read_byte(address),
            0x0400000C..=0x0400000D => self.bg_controls[2].read_byte(address),
            0x0400000E..=0x0400000F => self.bg_controls[3].read_byte(address),
            // BG0HOFS, BG0VOFS, BG1HOFS, BG1VOFS, BG2HOFS, BG2VOFS, BG3HOFS, BG3VOFS
            // BG2PA, BG2PB, BG2PC, BG2PD, BG2X_L, BG2X_H, BG2Y_L, BG2Y_H
            // BG3PA, BG3PB, BG3PC, BG3PD, BG3X_L, BG3X_H, BG3Y_L, BG3Y_H
            0x04000010..=0x0400003F => 0,
            _ => panic!("Invalid byte read for Background register: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
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
            0x04000020..=0x04000021 => self.bg_pa[0].write_byte(address, value),
            0x04000022..=0x04000023 => self.bg_pb[0].write_byte(address, value),
            0x04000024..=0x04000025 => self.bg_pc[0].write_byte(address, value),
            0x04000026..=0x04000027 => self.bg_pd[0].write_byte(address, value),
            // BG2X_L, BG2X_H, BG2Y_L, BG2Y_H
            0x04000028..=0x0400002B => {
                self.bg_x_references[0].write_byte(address, value);
                self.bg_x_affine[0] = self.bg_x_references[0].as_i32();
            }
            0x0400002C..=0x0400002F => {
                self.bg_y_references[0].write_byte(address, value);
                self.bg_y_affine[0] = self.bg_y_references[0].as_i32();
            }
            // BG3PA, BG3PB, BG3PC, BG3PD
            0x04000030..=0x04000031 => self.bg_pa[1].write_byte(address, value),
            0x04000032..=0x04000033 => self.bg_pb[1].write_byte(address, value),
            0x04000034..=0x04000035 => self.bg_pc[1].write_byte(address, value),
            0x04000036..=0x04000037 => self.bg_pd[1].write_byte(address, value),
            // BG3X_L, BG3X_H, BG3Y_L, BG3Y_H
            0x04000038..=0x0400003B => {
                self.bg_x_references[1].write_byte(address, value);
                self.bg_x_affine[1] = self.bg_x_references[1].as_i32();
            }
            0x0400003C..=0x0400003F => {
                self.bg_y_references[1].write_byte(address, value);
                self.bg_y_affine[1] = self.bg_y_references[1].as_i32();
            }
            _ => panic!("Invalid byte write for Background register: {:#010X}", address),
        }
    }
}
