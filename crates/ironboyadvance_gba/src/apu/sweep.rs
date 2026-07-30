use bitfields::{bitfield, bitflag};
use getset::CopyGetters;
use ironboyadvance_common::register_ops::RegisterOps;

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum SweepDirection {
    #[base]
    Increase = 0,
    Decrease = 1,
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct SweepControl {
    #[bits(3)]
    shift: u8,
    #[bits(1)]
    direction: SweepDirection,
    #[bits(3)]
    pace: u8,
    #[bits(9)]
    _not_used_7_15: u16,
}

impl RegisterOps<u16> for SweepControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }
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

        if self.timer == 0 {
            let pace = self.control.pace();
            match pace > 0 {
                true => self.timer = pace,
                false => self.timer = 8,
            }

            if self.enabled && pace > 0 {
                let new_period = self.calculate_period();

                if new_period <= 2047 && self.control.shift() > 0 {
                    self.shadow_period = new_period;
                    self.calculate_period();
                    return Some(new_period);
                }
            } else {
                self.period_calculated = false;
            }
        }

        None
    }

    fn calculate_period(&mut self) -> u16 {
        let offset = self.shadow_period >> self.control.shift();
        let new_period = match self.control.direction() {
            SweepDirection::Decrease => self.shadow_period - offset,
            SweepDirection::Increase => self.shadow_period + offset,
        };

        match new_period > 2047 {
            true => self.disable_channel = true,
            false => self.period_calculated = true,
        }
        new_period
    }

    pub fn read_8(&self, address: u32) -> u8 {
        self.control.read_byte(address)
    }

    pub fn write_8(&mut self, address: u32, value: u8) {
        let was_decreasing = self.control.direction() == SweepDirection::Decrease;
        self.control.write_byte(address, value);
        let is_decreasing = self.control.direction() == SweepDirection::Decrease;

        if was_decreasing && !is_decreasing && self.period_calculated {
            self.disable_channel = true;
        }
    }

    pub fn trigger(&mut self, frequency: u16) {
        self.shadow_period = frequency;

        let pace = self.control.pace();
        match pace > 0 {
            true => self.timer = pace,
            false => self.timer = 8,
        }

        self.enabled = pace > 0 || self.control.shift() > 0;
        self.disable_channel = false;

        if self.control.shift() > 0 {
            self.calculate_period();
        } else {
            self.period_calculated = false;
        }
    }
}
