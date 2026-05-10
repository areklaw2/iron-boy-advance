use bitfields::bitfield;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::io_registers::RegisterOps;

#[bitfield(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MosaicSize {
    #[bits(4)]
    bg_mosaic_h_size: u8, // (minus 1)
    #[bits(4)]
    bg_mosaic_v_size: u8, // (minus 1)
    #[bits(4)]
    obj_mosaic_h_size: u8, // (minus 1)
    #[bits(4)]
    obj_mosaic_v_size: u8, // (minus 1)
    _not_used_16_31: u16,
}

impl RegisterOps<u32> for MosaicSize {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.set_bits(bits);
    }
}

pub struct Mosaic {
    mosaic_size: MosaicSize,
}

impl Mosaic {
    pub fn new() -> Self {
        Self {
            mosaic_size: MosaicSize::from_bits(0),
        }
    }
}

impl SystemMemoryAccess for Mosaic {
    fn read_8(&self, _address: u32) -> u8 {
        0
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            0x0400004C..=0x0400004F => self.mosaic_size.write_byte(address, value),
            _ => panic!("Invalid byte write for Mosaic register: {:#010X}", address),
        }
    }
}
