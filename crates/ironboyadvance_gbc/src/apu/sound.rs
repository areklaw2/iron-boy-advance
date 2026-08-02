use bitfields::bitfield;

pub const MASTER_CONTROL_UNUSED_BITS: u8 = 0x70;

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct MasterVolume {
    #[bits(3)]
    right_volume: u8,
    vin_right_enable: bool,
    #[bits(3)]
    left_volume: u8,
    vin_left_enable: bool,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct SoundPanning {
    ch1_right_enable: bool,
    ch2_right_enable: bool,
    ch3_right_enable: bool,
    ch4_right_enable: bool,
    ch1_left_enable: bool,
    ch2_left_enable: bool,
    ch3_left_enable: bool,
    ch4_left_enable: bool,
}

impl SoundPanning {
    pub fn left_enabled(&self, channel: usize) -> bool {
        match channel {
            0 => self.ch1_left_enable(),
            1 => self.ch2_left_enable(),
            2 => self.ch3_left_enable(),
            _ => self.ch4_left_enable(),
        }
    }

    pub fn right_enabled(&self, channel: usize) -> bool {
        match channel {
            0 => self.ch1_right_enable(),
            1 => self.ch2_right_enable(),
            2 => self.ch3_right_enable(),
            _ => self.ch4_right_enable(),
        }
    }
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct MasterControl {
    ch1_on: bool,
    ch2_on: bool,
    ch3_on: bool,
    ch4_on: bool,
    #[bits(3)]
    _not_used_4_6: u8,
    enabled: bool,
}
