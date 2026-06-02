use bitfields::bitfield;
use getset::CopyGetters;
use ironboyadvance_common::memory::SystemMemoryAccess;

use ironboyadvance_common::register_ops::RegisterOps;

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct MosaicSize {
    #[bits(4)]
    bg_mosaic_h: u8, // (minus 1)
    #[bits(4)]
    bg_mosaic_v: u8, // (minus 1)
    #[bits(4)]
    obj_mosaic_h: u8, // (minus 1)
    #[bits(4)]
    obj_mosaic_v: u8, // (minus 1)
    _not_used_16_31: u16,
}

impl RegisterOps<u32> for MosaicSize {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.write_bits(bits);
    }
}

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Mosaic {
    size: MosaicSize,
    bg_source_y: u8,
    obj_source_y: u8,
}

impl Mosaic {
    pub fn new() -> Self {
        Self {
            size: MosaicSize::from_bits(0),
            bg_source_y: 0,
            obj_source_y: 0,
        }
    }

    // source_y is set when a mosiac block starts. If v_size changes mid-frame, the
    // in-progress block keeps its original source_y until the next boundary.
    pub fn update_sources(&mut self, v_count: u8) {
        if v_count == 0 {
            self.bg_source_y = 0;
            self.obj_source_y = 0;
            return;
        }

        let bg_v_size = self.size.bg_mosaic_v() + 1;
        let bg_mosaic_block_start = v_count.is_multiple_of(bg_v_size);
        if bg_mosaic_block_start {
            self.bg_source_y = v_count;
        }

        let obj_v_size = self.size.obj_mosaic_v() + 1;
        let obj_mosaic_block_start = v_count.is_multiple_of(obj_v_size);
        if obj_mosaic_block_start {
            self.obj_source_y = v_count;
        }
    }
}

impl SystemMemoryAccess for Mosaic {
    fn read_8(&self, _address: u32) -> u8 {
        0
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            0x0400004C..=0x0400004F => self.size.write_byte(address, value),
            _ => panic!("Invalid byte write for Mosaic register: {:#010X}", address),
        }
    }
}
