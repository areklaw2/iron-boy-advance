use bitfields::bitfield;
use getset::{CopyGetters, Setters};
use ironboyadvance_common::memory::SystemMemoryAccess;

use crate::apu::{
    length::{DEFAULT_MAX_LENGTH, Length},
    period::Period,
    sweep::Sweep,
    volume_envelope::VolumeEnvelope,
};

const LENGTH_UNUSED_BITS: u8 = 0x3F;
const PERIOD_HIGH_UNUSED_BITS: u8 = 0xBF;
const WRITE_ONLY: u8 = 0xFF;

const WAVEFORMS: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 0, 0],
];

const TICKS_PER_PERIOD_STEP: u16 = 4;

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct PulseLength {
    #[bits(6)]
    initial_length: u8,
    #[bits(2)]
    wave_duty: u8,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct PeriodHigh {
    #[bits(3)]
    frequency_high: u8,
    #[bits(3)]
    _not_used_3_5: u8,
    length_enable: bool,
    trigger: bool,
}

#[derive(Debug, CopyGetters, Setters)]
pub struct PulseChannel {
    #[getset(get_copy = "pub", set = "pub")]
    enabled: bool,
    dac_enabled: bool,
    wave_duty_position: u8,
    sweep: Option<Sweep>,
    length: Length,
    envelope: VolumeEnvelope,
    period: Period,
    control: PulseLength,
    period_low: u8,
    period_high: PeriodHigh,
    #[getset(set = "pub")]
    frame_sequencer_step: usize,
}

impl SystemMemoryAccess for PulseChannel {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF10 => match &self.sweep {
                Some(sweep) => sweep.read(),
                None => WRITE_ONLY,
            },
            0xFF11 | 0xFF16 => self.control.into_bits() | LENGTH_UNUSED_BITS,
            0xFF12 | 0xFF17 => self.envelope.read(),
            0xFF13 | 0xFF18 => WRITE_ONLY,
            0xFF14 | 0xFF19 => self.period_high.into_bits() | PERIOD_HIGH_UNUSED_BITS,
            _ => WRITE_ONLY,
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF10 => self.write_sweep(value),
            0xFF11 | 0xFF16 => self.write_length(value),
            0xFF12 | 0xFF17 => self.write_envelope(value),
            0xFF13 | 0xFF18 => self.period_low = value,
            0xFF14 | 0xFF19 => self.write_period_high(value),
            _ => {}
        }
    }
}

impl PulseChannel {
    pub fn new(with_sweep: bool) -> Self {
        PulseChannel {
            enabled: false,
            dac_enabled: false,
            wave_duty_position: 0,
            sweep: with_sweep.then(Sweep::new),
            length: Length::new(DEFAULT_MAX_LENGTH),
            envelope: VolumeEnvelope::new(),
            period: Period::new(),
            control: PulseLength::from_bits(0),
            period_low: 0,
            period_high: PeriodHigh::from_bits(0),
            frame_sequencer_step: 0,
        }
    }

    pub fn reset(&mut self, clear_length: bool) {
        self.enabled = false;
        self.dac_enabled = false;
        self.wave_duty_position = 0;
        if self.sweep.is_some() {
            self.sweep = Some(Sweep::new());
        }
        self.envelope = VolumeEnvelope::new();
        self.period = Period::new();
        self.control = PulseLength::from_bits(0);
        self.period_low = 0;
        self.period_high = PeriodHigh::from_bits(0);

        match clear_length {
            true => self.length = Length::new(DEFAULT_MAX_LENGTH),
            false => self.length.reset(),
        }
    }

    fn frequency(&self) -> u16 {
        (self.period_high.frequency_high() as u16) << 8 | self.period_low as u16
    }

    fn period_reload(&self) -> u16 {
        TICKS_PER_PERIOD_STEP * (2048 - self.frequency())
    }

    pub fn cycle(&mut self, ticks: usize) {
        if !self.enabled {
            return;
        }

        let steps = self.period.cycle(ticks, self.period_reload());
        self.wave_duty_position = ((self.wave_duty_position as usize + steps) % 8) as u8;
    }

    pub fn cycle_envelope(&mut self) {
        if self.enabled {
            self.envelope.cycle();
        }
    }

    pub fn cycle_length(&mut self) {
        if self.period_high.length_enable() {
            self.length.cycle();
            if self.length.expired() {
                self.enabled = false;
            }
        }
    }

    pub fn cycle_sweep(&mut self) {
        if !self.enabled {
            return;
        }

        let Some(sweep) = &mut self.sweep else {
            return;
        };

        let new_frequency = sweep.cycle();
        let disable = sweep.disable_channel();

        if let Some(frequency) = new_frequency {
            self.period_low = frequency as u8;
            self.period_high.set_frequency_high((frequency >> 8) as u8);
        }
        if disable {
            self.enabled = false;
        }
    }

    pub fn digital_output(&self) -> u8 {
        match self.enabled {
            true => WAVEFORMS[self.control.wave_duty() as usize][self.wave_duty_position as usize] * self.envelope.volume(),
            false => 0,
        }
    }

    fn trigger(&mut self) {
        if self.dac_enabled {
            self.enabled = true;
        }
        self.period.trigger(self.period_reload());
        self.length.reload();
        self.envelope.reset();

        let frequency = self.frequency();
        if let Some(sweep) = &mut self.sweep {
            sweep.trigger(frequency);
            if sweep.disable_channel() {
                self.enabled = false;
            }
        }
    }

    fn write_sweep(&mut self, value: u8) {
        if let Some(sweep) = &mut self.sweep {
            sweep.write(value);
            if sweep.disable_channel() {
                self.enabled = false;
            }
        }
    }

    fn write_length(&mut self, value: u8) {
        self.control = PulseLength::from_bits(value);
        self.length.set_initial_time(self.control.initial_length());
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

    fn write_period_high(&mut self, value: u8) {
        let was_length_enabled = self.period_high.length_enable();
        self.period_high = PeriodHigh::from_bits(value);

        let first_half = matches!(self.frame_sequencer_step, 1 | 3 | 5 | 7);
        if first_half && !was_length_enabled && self.period_high.length_enable() {
            self.cycle_length();
        }

        if self.period_high.trigger() {
            self.trigger();

            if first_half && self.period_high.length_enable() && self.length.maxxed() {
                self.cycle_length();
            }
        }
    }
}
