use bitfields::{bitfield, bitflag};
use getset::CopyGetters;

const SWEEP_UNUSED_BITS: u8 = 0x80;
const MAX_PERIOD: u16 = 2047;

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum SweepDirection {
    #[base]
    Increase = 0,
    Decrease = 1,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct SweepControl {
    #[bits(3)]
    step: u8,
    #[bits(1)]
    direction: SweepDirection,
    #[bits(3)]
    pace: u8,
    _not_used_7: bool,
}

#[derive(Debug, CopyGetters)]
pub struct Sweep {
    control: SweepControl,
    enabled: bool,
    shadow_period: u16,
    timer: u8,
    period_calculated: bool,
    #[getset(get_copy = "pub")]
    disable_channel: bool,
}

impl Sweep {
    pub fn new() -> Self {
        Sweep {
            control: SweepControl::from_bits(0),
            enabled: false,
            shadow_period: 0,
            timer: 0,
            period_calculated: false,
            disable_channel: false,
        }
    }

    pub fn cycle(&mut self) -> Option<u16> {
        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer != 0 {
            return None;
        }

        let pace = self.control.pace();
        self.timer = match pace > 0 {
            true => pace,
            false => 8,
        };

        if !self.enabled || pace == 0 {
            self.period_calculated = false;
            return None;
        }

        let new_period = self.calculate_period();
        if new_period <= MAX_PERIOD && self.control.step() > 0 {
            self.shadow_period = new_period;
            self.calculate_period();
            return Some(new_period);
        }

        None
    }

    fn calculate_period(&mut self) -> u16 {
        let offset = self.shadow_period >> self.control.step();
        let new_period = match self.control.direction() {
            SweepDirection::Decrease => self.shadow_period.wrapping_sub(offset),
            SweepDirection::Increase => self.shadow_period + offset,
        };

        match new_period > MAX_PERIOD {
            true => self.disable_channel = true,
            false => self.period_calculated = true,
        }
        new_period
    }

    pub fn read(&self) -> u8 {
        self.control.into_bits() | SWEEP_UNUSED_BITS
    }

    pub fn write(&mut self, value: u8) {
        let was_decreasing = self.control.direction() == SweepDirection::Decrease;
        self.control = SweepControl::from_bits(value);
        let is_decreasing = self.control.direction() == SweepDirection::Decrease;

        if was_decreasing && !is_decreasing && self.period_calculated {
            self.disable_channel = true;
        }
    }

    pub fn trigger(&mut self, frequency: u16) {
        self.shadow_period = frequency;

        let pace = self.control.pace();
        self.timer = match pace > 0 {
            true => pace,
            false => 8,
        };

        self.enabled = pace > 0 || self.control.step() > 0;
        self.disable_channel = false;

        match self.control.step() > 0 {
            true => {
                self.calculate_period();
            }
            false => self.period_calculated = false,
        }
    }
}
