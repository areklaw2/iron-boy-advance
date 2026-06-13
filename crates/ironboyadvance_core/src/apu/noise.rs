use crate::apu::length::{DEFAULT_MAX_LENGTH, Length};
use crate::apu::period::Period;
use crate::apu::volume_envelope::{EnvelopeDirection, VolumeEnvelope};
use bitfields::{bitfield, bitflag};
use getset::{CopyGetters, Setters};
use ironboyadvance_common::memory::SystemMemoryAccess;
use ironboyadvance_common::register_ops::RegisterOps;

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct NoiseControl {
    #[bits(6)]
    length: u8,
    #[bits(2)]
    _not_used_6_7: u8,
    #[bits(3)]
    envelope_pace: u8,
    #[bits(1)]
    envelope_direction: EnvelopeDirection,
    #[bits(4)]
    initial_volume: u8,
    _not_used_16_31: u16,
}

impl RegisterOps<u32> for NoiseControl {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.write_bits(bits);
    }

    fn read_mask(&self) -> u32 {
        0xFFFF_FFC0
    }
}

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum CounterWidth {
    #[base]
    FullWidth = 0x0, //15 bit
    NarrowWidth = 0x1, //7 bit
}

impl CounterWidth {
    fn width(&self) -> u16 {
        match self {
            CounterWidth::FullWidth => 15,
            CounterWidth::NarrowWidth => 7,
        }
    }

    fn step(&self, lfsr: u16) -> u16 {
        let feedback = !((lfsr ^ (lfsr >> 1)) & 1) & 1;
        let tap = self.width() - 1; // 14 or 6
        let shifted = lfsr >> 1;
        (shifted & !(1 << tap)) | (feedback << tap)
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct NoiseFrequency {
    #[bits(3)]
    divider: u8,
    #[bits(1)]
    counter_step: CounterWidth,
    #[bits(4)]
    shift_clock: u8,
    #[bits(6)]
    _not_used_8_13: u8,
    length_enable: bool,
    trigger: bool,
    _not_used_16_31: u16,
}

impl RegisterOps<u32> for NoiseFrequency {
    fn register(&self) -> u32 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u32) {
        self.write_bits(bits);
    }

    fn read_mask(&self) -> u32 {
        0xFFFF_7800
    }
}

#[derive(Debug, CopyGetters, Setters)]
pub struct NoiseChannel {
    #[getset(get_copy = "pub", set = "pub")]
    enabled: bool,
    dac_enabled: bool,
    length: Length,
    envelope: VolumeEnvelope,
    lfsr: u16,
    period: Period,
    control: NoiseControl,
    frequency: NoiseFrequency,
}

impl SystemMemoryAccess for NoiseChannel {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // SOUND4CNT_L
            0x04000078..=0x0400007B => self.control.read_byte(address),
            // SOUND4CNT_H
            0x0400007C..=0x0400007F => self.frequency.read_byte(address),
            _ => 0,
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // SOUND4CNT_L
            0x04000078..=0x0400007B => self.write_control(address, value),
            // SOUND4CNT_H
            0x0400007C..=0x0400007F => self.write_frequency(address, value),
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
            lfsr: 0,
            period: Period::new(),
            control: NoiseControl::from_bits(0),
            frequency: NoiseFrequency::from_bits(0),
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.dac_enabled = false;
        self.envelope = VolumeEnvelope::new();
        self.lfsr = 0;
        self.period = Period::new();
        self.control = NoiseControl::from_bits(0);
        self.frequency = NoiseFrequency::from_bits(0);
        self.length.reset();
    }

    fn period_cycles(&self) -> usize {
        let clock_shift = self.frequency.shift_clock();
        let clock_divider = self.frequency.divider();
        let divisor = if clock_divider == 0 {
            8
        } else {
            (clock_divider as usize) << 4
        };
        (divisor << clock_shift) << 2
    }

    pub fn cycle(&mut self, cycles: usize) {
        if !self.enabled {
            return;
        }

        let steps = self.period.step(cycles, self.period_cycles());
        for _ in 0..steps {
            self.lfsr = self.frequency.counter_step().step(self.lfsr);
        }
    }

    pub fn cycle_envelope(&mut self) {
        if self.enabled {
            self.envelope.cycle();
        }
    }

    pub fn cycle_length(&mut self) {
        if self.frequency.length_enable() {
            self.length.cycle();
            if self.length.expired() {
                self.set_enabled(false);
            }
        }
    }

    pub fn dac_output(&self) -> f32 {
        if self.enabled {
            let amplitude = (self.lfsr & 0x01) as u8;
            let digital = amplitude * self.envelope.volume();
            (digital as f32 / 7.5) - 1.0
        } else {
            0.0
        }
    }

    pub fn trigger(&mut self) {
        if self.dac_enabled {
            self.enabled = true;
        }
        self.period.trigger();
        self.lfsr = 0;
        self.length.reload();
        self.envelope.reset();
    }

    fn write_control(&mut self, address: u32, value: u8) {
        self.control.write_byte(address, value);

        match address & 1 == 0 {
            true => {
                self.length.set_initial_time(self.control.length());
                self.length.initialize();
            }
            false => {
                self.envelope.set_initial_volume(self.control.initial_volume());
                self.envelope.set_pace(self.control.envelope_pace());
                self.envelope.set_direction(self.control.envelope_direction());
                self.envelope.reset();

                self.dac_enabled = !self.envelope.disable_dac();
                if !self.dac_enabled {
                    self.enabled = false;
                }
            }
        }
    }

    fn write_frequency(&mut self, address: u32, value: u8) {
        self.frequency.write_byte(address, value);

        // Trigger (bit 15) lives in the high byte of the low 16-bit register.
        // TODO: obscure length-counter clock on trigger/enable needs the frame-seq step.
        if address & 3 == 1 && self.frequency.trigger() {
            self.trigger();
            self.frequency.set_trigger(false);
        }
    }
}
