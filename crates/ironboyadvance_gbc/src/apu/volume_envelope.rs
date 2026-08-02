use bitfields::{bitfield, bitflag};
use getset::CopyGetters;

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum EnvelopeDirection {
    #[base]
    Decrease = 0,
    Increase = 1,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct EnvelopeControl {
    #[bits(3)]
    pace: u8,
    #[bits(1)]
    direction: EnvelopeDirection,
    #[bits(4)]
    initial_volume: u8,
}

#[derive(Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct VolumeEnvelope {
    control: EnvelopeControl,
    volume: u8,
    timer: u8,
}

impl VolumeEnvelope {
    pub fn new() -> Self {
        VolumeEnvelope {
            control: EnvelopeControl::from_bits(0),
            volume: 0,
            timer: 0,
        }
    }

    pub fn cycle(&mut self) {
        let pace = self.control.pace();
        if pace == 0 {
            return;
        }

        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.timer = pace;
            match self.control.direction() {
                EnvelopeDirection::Increase if self.volume < 0xF => self.volume += 1,
                EnvelopeDirection::Decrease if self.volume > 0x0 => self.volume -= 1,
                _ => {}
            }
        }
    }

    pub fn reset(&mut self) {
        self.timer = self.control.pace();
        self.volume = self.control.initial_volume();
    }

    pub fn read(&self) -> u8 {
        self.control.into_bits()
    }

    pub fn write(&mut self, value: u8) {
        self.control = EnvelopeControl::from_bits(value);
    }

    pub fn disable_dac(&self) -> bool {
        self.control.initial_volume() == 0 && self.control.direction() == EnvelopeDirection::Decrease
    }
}
