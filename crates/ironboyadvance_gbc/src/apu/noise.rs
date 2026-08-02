use bitfields::{bitfield, bitflag};
use getset::{CopyGetters, Setters};
use ironboyadvance_common::{bits::BitOps, memory::SystemMemoryAccess};

use crate::apu::{
    length::{DEFAULT_MAX_LENGTH, Length},
    period::Period,
    volume_envelope::VolumeEnvelope,
};

const CONTROL_UNUSED_BITS: u8 = 0xBF;
const WRITE_ONLY: u8 = 0xFF;

const DIVISORS: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum CounterWidth {
    #[base]
    FullWidth = 0,
    NarrowWidth = 1,
}

impl CounterWidth {
    fn tap(&self) -> usize {
        match self {
            CounterWidth::FullWidth => 14,
            CounterWidth::NarrowWidth => 6,
        }
    }
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Polynomial {
    #[bits(3)]
    divider: u8,
    #[bits(1)]
    counter_width: CounterWidth,
    #[bits(4)]
    shift_clock: u8,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct NoiseLength {
    #[bits(6)]
    initial_length: u8,
    #[bits(2)]
    _not_used_6_7: u8,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct NoiseControl {
    #[bits(6)]
    _not_used_0_5: u8,
    length_enable: bool,
    trigger: bool,
}

#[derive(Debug, CopyGetters, Setters)]
pub struct NoiseChannel {
    #[getset(get_copy = "pub", set = "pub")]
    enabled: bool,
    dac_enabled: bool,
    length: Length,
    envelope: VolumeEnvelope,
    period: Period,
    lfsr: u16,
    polynomial: Polynomial,
    control: NoiseControl,
    #[getset(set = "pub")]
    frame_sequencer_step: usize,
}

impl SystemMemoryAccess for NoiseChannel {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF20 => WRITE_ONLY,
            0xFF21 => self.envelope.read(),
            0xFF22 => self.polynomial.into_bits(),
            0xFF23 => self.control.into_bits() | CONTROL_UNUSED_BITS,
            _ => WRITE_ONLY,
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF20 => self.write_length(value),
            0xFF21 => self.write_envelope(value),
            0xFF22 => self.polynomial = Polynomial::from_bits(value),
            0xFF23 => self.write_control(value),
            _ => {}
        }
    }
}

impl NoiseChannel {
    pub fn new() -> Self {
        NoiseChannel {
            enabled: false,
            dac_enabled: false,
            length: Length::new(DEFAULT_MAX_LENGTH),
            envelope: VolumeEnvelope::new(),
            period: Period::new(),
            lfsr: 0,
            polynomial: Polynomial::from_bits(0),
            control: NoiseControl::from_bits(0),
            frame_sequencer_step: 0,
        }
    }

    pub fn reset(&mut self, clear_length: bool) {
        self.enabled = false;
        self.dac_enabled = false;
        self.envelope = VolumeEnvelope::new();
        self.period = Period::new();
        self.lfsr = 0;
        self.polynomial = Polynomial::from_bits(0);
        self.control = NoiseControl::from_bits(0);

        match clear_length {
            true => self.length = Length::new(DEFAULT_MAX_LENGTH),
            false => self.length.reset(),
        }
    }

    fn period_reload(&self) -> u16 {
        DIVISORS[self.polynomial.divider() as usize] << self.polynomial.shift_clock()
    }

    pub fn cycle(&mut self, ticks: usize) {
        if !self.enabled {
            return;
        }

        let steps = self.period.cycle(ticks, self.period_reload());
        for _ in 0..steps {
            let feedback = !(self.lfsr.bit(0) ^ self.lfsr.bit(1));
            self.lfsr >>= 1;
            self.lfsr.set_bit(self.polynomial.counter_width().tap(), feedback);
        }
    }

    pub fn cycle_envelope(&mut self) {
        if self.enabled {
            self.envelope.cycle();
        }
    }

    pub fn cycle_length(&mut self) {
        if self.control.length_enable() {
            self.length.cycle();
            if self.length.expired() {
                self.enabled = false;
            }
        }
    }

    pub fn digital_output(&self) -> u8 {
        match self.enabled {
            true => (self.lfsr & 0x01) as u8 * self.envelope.volume(),
            false => 0,
        }
    }

    fn trigger(&mut self) {
        if self.dac_enabled {
            self.enabled = true;
        }
        self.period.trigger(self.period_reload());
        self.lfsr = 0;
        self.length.reload();
        self.envelope.reset();
    }

    fn write_length(&mut self, value: u8) {
        self.length.set_initial_time(NoiseLength::from_bits(value).initial_length());
        self.length.initialize();
    }

    fn write_envelope(&mut self, value: u8) {
        self.envelope.write(value);
        self.envelope.reset();

        self.dac_enabled = !self.envelope.disable_dac();
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    fn write_control(&mut self, value: u8) {
        let was_length_enabled = self.control.length_enable();
        self.control = NoiseControl::from_bits(value);

        let first_half = matches!(self.frame_sequencer_step, 1 | 3 | 5 | 7);
        if first_half && !was_length_enabled && self.control.length_enable() {
            self.cycle_length();
        }

        if self.control.trigger() {
            self.trigger();

            if first_half && self.control.length_enable() && self.length.maxxed() {
                self.cycle_length();
            }
        }
    }
}
