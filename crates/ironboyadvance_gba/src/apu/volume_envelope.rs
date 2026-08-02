use bitfields::bitflag;
use getset::{CopyGetters, Setters};

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum EnvelopeDirection {
    #[base]
    Decrease = 0,
    Increase = 1,
}

#[derive(Debug, CopyGetters, Setters)]
#[getset(get_copy = "pub", set = "pub")]
pub struct VolumeEnvelope {
    initial_volume: u8,
    direction: EnvelopeDirection,
    pace: u8,
    volume: u8,
    timer: u8,
}

impl VolumeEnvelope {
    pub fn new() -> Self {
        VolumeEnvelope {
            initial_volume: 0,
            direction: EnvelopeDirection::Decrease,
            pace: 0,
            volume: 0,
            timer: 0,
        }
    }

    pub fn cycle(&mut self) {
        if self.pace == 0 {
            return;
        }

        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.timer = self.pace;
            match self.direction {
                EnvelopeDirection::Increase if self.volume < 0xF => self.volume += 1,
                EnvelopeDirection::Decrease if self.volume > 0x0 => self.volume -= 1,
                _ => {}
            }
        }
    }

    pub fn reset(&mut self) {
        self.timer = self.pace;
        self.volume = self.initial_volume;
    }

    pub fn disable_dac(&self) -> bool {
        self.initial_volume == 0 && self.direction == EnvelopeDirection::Decrease
    }
}
