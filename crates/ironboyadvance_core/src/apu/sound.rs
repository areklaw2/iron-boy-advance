use bitfields::{bitfield, bitflag};
use ironboyadvance_common::register_ops::RegisterOps;

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct PsgSoundControl {
    #[bits(3)]
    right_volume: u8,
    _not_used_3: bool,
    #[bits(3)]
    left_volume: u8,
    _not_used_7: bool,
    ch1_right_enable: bool,
    ch2_right_enable: bool,
    ch3_right_enable: bool,
    ch4_right_enable: bool,
    ch1_left_enable: bool,
    ch2_left_enable: bool,
    ch3_left_enable: bool,
    ch4_left_enable: bool,
}

impl RegisterOps<u16> for PsgSoundControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }
}

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum PsgVolumeRatio {
    #[base]
    Ratio25 = 0x0,
    Ratio50 = 0x1,
    Ratio100 = 0x2,
    Prohibited = 0x3,
}

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum DmaVolumeRatio {
    #[base]
    Ratio50 = 0x0,
    Ratio100 = 0x1,
}

impl DmaVolumeRatio {
    pub fn scale(&self) -> f32 {
        match self {
            DmaVolumeRatio::Ratio50 => 0.5,
            DmaVolumeRatio::Ratio100 => 1.0,
        }
    }
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct DmaSoundControl {
    #[bits(2)]
    psg_volume_ratio: PsgVolumeRatio,
    #[bits(1)]
    dma_a_volume_ratio: DmaVolumeRatio,
    #[bits(1)]
    dma_b_volume_ratio: DmaVolumeRatio,
    #[bits(4)]
    _not_used_4_7: u8,
    dma_a_right_enable: bool,
    dma_a_left_enable: bool,
    // false = Timer 0, true = Timer 1.
    dma_a_timer_select: bool,
    dma_a_reset_fifo: bool,
    dma_b_right_enable: bool,
    dma_b_left_enable: bool,
    dma_b_timer_select: bool,
    dma_b_reset_fifo: bool,
}

impl RegisterOps<u16> for DmaSoundControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }

    fn read_mask(&self) -> u16 {
        0x77FF
    }
}

impl DmaSoundControl {
    pub fn dma_a_active(&self, timer_id: usize) -> bool {
        self.dma_a_timer_select() as usize == timer_id && (self.dma_a_left_enable() || self.dma_a_right_enable())
    }

    pub fn dma_b_active(&self, timer_id: usize) -> bool {
        self.dma_b_timer_select() as usize == timer_id && (self.dma_b_left_enable() || self.dma_b_right_enable())
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct SoundStatus {
    ch1_on: bool,
    ch2_on: bool,
    ch3_on: bool,
    ch4_on: bool,
    #[bits(3)]
    _not_used_4_6: u8,
    master_enable: bool,
    #[bits(24)]
    _not_used_8_31: u32,
}

impl RegisterOps<u32> for SoundStatus {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.write_bits(bits);
    }

    fn write_mask(&self) -> u32 {
        0xFFFF_FFF0
    }
}

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum AmplitudeResolution {
    #[base]
    Nine = 0x0, //9bit
    Eight = 0x1, //8bit
    Seven = 0x2, //7bit
    Six = 0x3,   //6bit
}

impl AmplitudeResolution {
    pub fn sampling_frequency(&self) -> usize {
        match self {
            AmplitudeResolution::Nine => 32768,
            AmplitudeResolution::Eight => 65536,
            AmplitudeResolution::Seven => 131072,
            AmplitudeResolution::Six => 262144,
        }
    }
}

#[bitfield(u32)]
pub struct SoundBias {
    _not_used_0: bool,
    #[bits(9)]
    bias_level: u16,
    #[bits(4)]
    _not_used_10_13: u8,
    #[bits(2)]
    amplitude_resolution: AmplitudeResolution,
    #[bits(16)]
    _not_used_16_31: u32,
}

impl RegisterOps<u32> for SoundBias {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.write_bits(bits);
    }
}
