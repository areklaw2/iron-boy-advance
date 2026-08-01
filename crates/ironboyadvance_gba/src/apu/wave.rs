use crate::apu::{
    length::{Length, WAVE_MAX_LENGTH},
    period::Period,
};
use bitfields::{bitfield, bitflag};
use getset::{CopyGetters, Setters};
use ironboyadvance_common::{memory::SystemMemoryAccess, register_ops::RegisterOps};

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum WaveDimension {
    #[base]
    OneBank = 0x0, // 32 samples
    TwoBanks = 0x1, // 64 samples
}

impl WaveDimension {
    fn sample_count(&self) -> usize {
        match self {
            WaveDimension::OneBank => 32,
            WaveDimension::TwoBanks => 64,
        }
    }
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct WaveControl {
    #[bits(5)]
    _not_used_0_4: u8,
    #[bits(1)]
    dimension: WaveDimension,
    bank: bool,
    dac_enabled: bool,
    _not_used_8_15: u8,
}

impl RegisterOps<u16> for WaveControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }
}

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum VolumeLevel {
    #[base]
    Mute = 0, // 0%
    Full = 1,    // 100%
    Half = 2,    // 50%
    Quarter = 3, // 25%
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct WaveVolume {
    #[bits(8)]
    length: u8,
    #[bits(5)]
    _not_used_8_12: u8,
    #[bits(2)]
    level: VolumeLevel,
    force_volume: bool,
}

impl RegisterOps<u16> for WaveVolume {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.write_bits(bits);
    }

    fn read_mask(&self) -> u16 {
        0xFF00
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct WaveFrequency {
    #[bits(11)]
    sample_rate: u16,
    #[bits(3)]
    _not_used_11_13: u8,
    length_enable: bool,
    trigger: bool,
    _not_used_16_31: u16,
}

impl RegisterOps<u32> for WaveFrequency {
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
pub struct WaveChannel {
    #[getset(get_copy = "pub", set = "pub")]
    enabled: bool,
    length: Length,
    period: Period,
    wave_position: u8,
    wave_ram: [[u8; 16]; 2],
    control: WaveControl,
    volume: WaveVolume,
    frequency: WaveFrequency,
    #[getset(set = "pub")]
    frame_sequencer_step: usize,
}

impl SystemMemoryAccess for WaveChannel {
    type Address = u32;

    fn read_8(&self, address: u32) -> u8 {
        match address {
            0x04000070..=0x04000071 => self.control.read_byte(address),
            0x04000072..=0x04000073 => self.volume.read_byte(address),
            0x04000074..=0x04000077 => self.frequency.read_byte(address),
            0x04000090..=0x0400009F => self.read_wave_ram(address),
            _ => 0,
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            0x04000070..=0x04000071 => self.write_control(address, value),
            0x04000072..=0x04000073 => self.volume.write_byte(address, value),
            0x04000074..=0x04000077 => self.write_frequency(address, value),
            0x04000090..=0x0400009F => self.write_wave_ram(address, value),
            _ => {}
        }
    }
}

impl WaveChannel {
    pub fn new() -> Self {
        WaveChannel {
            enabled: false,
            length: Length::new(WAVE_MAX_LENGTH),
            period: Period::new(),
            wave_position: 0,
            wave_ram: [[0; 16]; 2],
            control: WaveControl::from_bits(0),
            volume: WaveVolume::from_bits(0),
            frequency: WaveFrequency::from_bits(0),
            frame_sequencer_step: 0,
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.period = Period::new();
        self.wave_position = 0;
        self.control = WaveControl::from_bits(0);
        self.volume = WaveVolume::from_bits(0);
        self.frequency = WaveFrequency::from_bits(0);
        self.length.reset();
    }

    fn period_cycles(&self) -> usize {
        8 * (2048 - self.frequency.sample_rate() as usize)
    }

    pub fn cycle(&mut self, cycles: usize) {
        if !self.enabled {
            return;
        }

        let steps = self.period.step(cycles, self.period_cycles());
        let sample_count = self.control.dimension().sample_count();
        self.wave_position = ((self.wave_position as usize + steps) % sample_count) as u8;
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
            let position = self.wave_position;
            let two_banks = self.control.dimension() == WaveDimension::TwoBanks;
            let (bank, nibble_index) = match two_banks && position >= 32 {
                true => (self.control.bank() as usize ^ 1, position - 32),
                false => (self.control.bank() as usize, position),
            };

            let byte = self.wave_ram[bank][(nibble_index / 2) as usize];
            let sample = match nibble_index % 2 == 0 {
                true => byte >> 4,
                false => byte & 0x0F,
            };

            let scaled = match self.volume.force_volume() {
                true => (sample as u16 * 3 / 4) as u8,
                false => match self.volume.level() {
                    VolumeLevel::Mute => 0,
                    VolumeLevel::Full => sample,
                    VolumeLevel::Half => sample >> 1,
                    VolumeLevel::Quarter => sample >> 2,
                },
            };
            (scaled as f32 / 7.5) - 1.0
        } else {
            0.0
        }
    }

    pub fn trigger(&mut self) {
        self.wave_position = 0;
        if self.control.dac_enabled() {
            self.enabled = true;
        }
        self.period.trigger();
        self.length.reload();
    }

    fn write_control(&mut self, address: u32, value: u8) {
        self.control.write_byte(address, value);
        if !self.control.dac_enabled() {
            self.enabled = false;
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

    fn read_wave_ram(&self, address: u32) -> u8 {
        let bank = self.control.bank() as usize ^ 1;
        self.wave_ram[bank][(address & 0x0F) as usize]
    }

    fn write_wave_ram(&mut self, address: u32, value: u8) {
        let bank = self.control.bank() as usize ^ 1;
        self.wave_ram[bank][(address & 0x0F) as usize] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triggered_wave() -> WaveChannel {
        let mut wave = WaveChannel::new();
        // Select bank 1 for playback so CPU writes target the OTHER bank (bank 0).
        wave.write_8(0x04000070, 0xC0); // dac on (bit 7) + bank = 1 (bit 6)
        // Fill the idle bank with alternating nibbles: 15, 0, 15, 0, ...
        for offset in 0..16 {
            wave.write_8(0x04000090 + offset, 0xF0);
        }
        // Switch playback to bank 0 (the bank we just filled); DAC stays on.
        wave.write_8(0x04000070, 0x80);
        // SOUND3CNT_H high byte: volume level = Full (bit 13).
        wave.write_8(0x04000073, 0x20);
        // SOUND3CNT_X: sample rate 2045 (step coprime to 32 -> full coverage), trigger.
        wave.write_8(0x04000074, 0xFD);
        wave.write_8(0x04000075, 0x87);
        wave
    }

    #[test]
    fn trigger_enables_channel() {
        assert!(triggered_wave().enabled());
    }

    #[test]
    fn plays_loaded_sample() {
        // Right after trigger, position 0 reads bank 0's first high nibble (15).
        // Proves the double-buffer routing (CPU wrote bank 0 while bank 1 was selected)
        // and full-volume scaling: (15 / 7.5) - 1.0 == 1.0.
        let wave = triggered_wave();
        assert_eq!(wave.dac_output(), 1.0);
    }

    #[test]
    fn produces_varying_output() {
        let mut wave = triggered_wave();
        let mut saw_low = false;
        let mut saw_high = false;
        for _ in 0..256 {
            wave.cycle(512);
            match wave.dac_output() {
                out if out < 0.0 => saw_low = true,
                out if out > 0.0 => saw_high = true,
                _ => {}
            }
        }
        assert!(
            saw_low && saw_high,
            "wave output should swing both ways: low={saw_low} high={saw_high}"
        );
    }

    #[test]
    fn silent_when_disabled() {
        let wave = WaveChannel::new();
        assert!(!wave.enabled());
        assert_eq!(wave.dac_output(), 0.0);
    }
}
