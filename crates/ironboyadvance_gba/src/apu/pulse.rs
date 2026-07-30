use crate::apu::length::{DEFAULT_MAX_LENGTH, Length};
use crate::apu::period::Period;
use crate::apu::sweep::Sweep;
use crate::apu::volume_envelope::{EnvelopeDirection, VolumeEnvelope};

use bitfields::bitfield;
use getset::{CopyGetters, Setters};
use ironboyadvance_common::memory::SystemMemoryAccess;
use ironboyadvance_common::register_ops::RegisterOps;

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct PulseControl {
    #[bits(6)]
    length: u8,
    #[bits(2)]
    wave_duty: u8,
    #[bits(3)]
    envelope_pace: u8,
    #[bits(1)]
    envelope_direction: EnvelopeDirection,
    #[bits(4)]
    initial_volume: u8,
}

impl RegisterOps<u16> for PulseControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }

    fn read_mask(&self) -> u16 {
        0xFFC0
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct PulseFrequency {
    #[bits(11)]
    frequency: u16,
    #[bits(3)]
    _not_used_11_13: u8,
    length_enable: bool,
    trigger: bool,
    _not_used_16_31: u16,
}

impl RegisterOps<u32> for PulseFrequency {
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
pub struct PulseChannel {
    #[getset(get_copy = "pub", set = "pub")]
    enabled: bool,
    dac_enabled: bool,
    wave_duty_position: u8,
    sweep: Option<Sweep>,
    length: Length,
    envelope: VolumeEnvelope,
    period: Period,
    control: PulseControl,
    frequency: PulseFrequency,
    #[getset(set = "pub")]
    frame_sequencer_step: usize,
}

const WAVEFORMS: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 0, 0],
];

// CPU cycles per duty step: 16 * (2048 - frequency).
const PERIOD_TICK_CYCLES: usize = 16;

impl SystemMemoryAccess for PulseChannel {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // SOUND1CNT_L
            0x04000060..=0x04000061 => match &self.sweep {
                Some(sweep) => sweep.read_8(address),
                None => 0,
            },
            // SOUND1CNT_H / SOUND2CNT_L
            0x04000062..=0x04000063 | 0x04000068..=0x04000069 => self.control.read_byte(address),
            // SOUND1CNT_X / SOUND2CNT_H
            0x04000064..=0x04000067 | 0x0400006C..=0x0400006F => self.frequency.read_byte(address),
            _ => 0,
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // SOUND1CNT_L
            0x04000060..=0x04000061 => {
                if let Some(sweep) = &mut self.sweep {
                    sweep.write_8(address, value);
                    if sweep.disable_channel() {
                        self.enabled = false;
                    }
                }
            }
            // SOUND1CNT_H / SOUND2CNT_L
            0x04000062..=0x04000063 | 0x04000068..=0x04000069 => self.write_control(address, value),
            // SOUND1CNT_X / SOUND2CNT_H
            0x04000064..=0x04000067 | 0x0400006C..=0x0400006F => self.write_frequency(address, value),
            _ => {}
        }
    }
}

impl PulseChannel {
    pub fn new(with_sweep: bool) -> Self {
        let sweep = match with_sweep {
            true => Some(Sweep::new()),
            false => None,
        };

        PulseChannel {
            enabled: false,
            dac_enabled: false,
            wave_duty_position: 0,
            sweep,
            length: Length::new(DEFAULT_MAX_LENGTH),
            envelope: VolumeEnvelope::new(),
            period: Period::new(),
            control: PulseControl::from_bits(0),
            frequency: PulseFrequency::from_bits(0),
            frame_sequencer_step: 0,
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.dac_enabled = false;
        self.wave_duty_position = 0;
        if self.sweep.is_some() {
            self.sweep = Some(Sweep::new());
        }
        self.length.reset();
        self.envelope = VolumeEnvelope::new();
        self.period = Period::new();
        self.control = PulseControl::from_bits(0);
        self.frequency = PulseFrequency::from_bits(0);
    }

    pub fn cycle(&mut self, cycles: usize) {
        if !self.enabled {
            return;
        }

        let period_cycles = PERIOD_TICK_CYCLES * (2048 - self.frequency.frequency() as usize);
        let steps = self.period.step(cycles, period_cycles);
        self.wave_duty_position = ((self.wave_duty_position as usize + steps) % 8) as u8;
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

    pub fn cycle_sweep(&mut self) {
        if !self.enabled {
            return;
        }

        let (new_frequency, disable) = match &mut self.sweep {
            Some(sweep) => (sweep.cycle(), sweep.disable_channel()),
            None => return,
        };

        if let Some(frequency) = new_frequency {
            self.frequency.set_frequency(frequency);
        }
        if disable {
            self.set_enabled(false);
        }
    }

    pub fn dac_output(&self) -> f32 {
        if self.enabled {
            let digital =
                WAVEFORMS[self.control.wave_duty() as usize][self.wave_duty_position as usize] * self.envelope.volume();
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
        self.length.reload();
        self.envelope.reset();

        let frequency = self.frequency.frequency();
        if let Some(sweep) = &mut self.sweep {
            sweep.trigger(frequency);
            if sweep.disable_channel() {
                self.set_enabled(false);
            }
        }
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
        let was_length_enabled = self.frequency.length_enable();
        self.frequency.write_byte(address, value);

        let first_half = matches!(self.frame_sequencer_step, 1 | 3 | 5 | 7);
        if first_half && !was_length_enabled && self.frequency.length_enable() {
            self.cycle_length();
        }

        if address & 3 == 1 && self.frequency.trigger() {
            self.trigger();
            self.frequency.set_trigger(false);

            if first_half && self.frequency.length_enable() && self.length.maxxed() {
                self.cycle_length();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Channel 2 (no sweep), registers at 0x68-0x6F.
    fn triggered_pulse() -> PulseChannel {
        let mut pulse = PulseChannel::new(false);
        // SOUND2CNT_L low byte: 50% duty (bits 6-7 = 0b10).
        pulse.write_8(0x04000068, 0x80);
        // SOUND2CNT_L high byte: initial_volume = 15 -> DAC enabled.
        pulse.write_8(0x04000069, 0xF0);
        // SOUND2CNT_H: frequency 2040 (period 128 cyc), then trigger (bit 15).
        pulse.write_8(0x0400006C, 0xF8);
        pulse.write_8(0x0400006D, 0x87);
        pulse
    }

    #[test]
    fn trigger_enables_channel() {
        assert!(triggered_pulse().enabled());
    }

    #[test]
    fn produces_a_square_wave() {
        let mut pulse = triggered_pulse();
        let mut saw_low = false;
        let mut saw_high = false;
        for _ in 0..256 {
            pulse.cycle(512);
            match pulse.dac_output() {
                out if out < 0.0 => saw_low = true,
                out if out > 0.0 => saw_high = true,
                _ => {}
            }
        }
        assert!(
            saw_low && saw_high,
            "duty wave should swing both ways: low={saw_low} high={saw_high}"
        );
    }

    #[test]
    fn silent_when_disabled() {
        let pulse = PulseChannel::new(false);
        assert!(!pulse.enabled());
        assert_eq!(pulse.dac_output(), 0.0);
    }
}
