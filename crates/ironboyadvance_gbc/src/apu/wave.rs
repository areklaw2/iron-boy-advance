use bitfields::{bitfield, bitflag};
use getset::{CopyGetters, Setters};
use ironboyadvance_common::memory::SystemMemoryAccess;
use ironboyadvance_sm83::GbMode;

use crate::apu::{
    length::{Length, WAVE_MAX_LENGTH},
    period::Period,
};

const DAC_UNUSED_BITS: u8 = 0x7F;
const VOLUME_UNUSED_BITS: u8 = 0x9F;
const PERIOD_HIGH_UNUSED_BITS: u8 = 0xBF;
const WRITE_ONLY: u8 = 0xFF;

const WAVE_RAM_SIZE: usize = 0x10;
const WAVE_SAMPLES: u8 = 32;
const TICKS_PER_PERIOD_STEP: u16 = 2;

#[bitflag(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum VolumeLevel {
    #[base]
    Mute = 0,
    Full = 1,
    Half = 2,
    Quarter = 3,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct DacControl {
    #[bits(7)]
    _not_used_0_6: u8,
    enabled: bool,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct WaveVolume {
    #[bits(5)]
    _not_used_0_4: u8,
    #[bits(2)]
    level: VolumeLevel,
    _not_used_7: bool,
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
pub struct WaveChannel {
    #[getset(get_copy = "pub", set = "pub")]
    enabled: bool,
    dac_control: DacControl,
    length: Length,
    period: Period,
    wave_position: u8,
    wave_ram: Vec<u8>,
    volume: WaveVolume,
    period_low: u8,
    period_high: PeriodHigh,
    #[getset(set = "pub")]
    frame_sequencer_step: usize,
    mode: GbMode,
}

impl SystemMemoryAccess for WaveChannel {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF1A => self.dac_control.into_bits() | DAC_UNUSED_BITS,
            0xFF1B => WRITE_ONLY,
            0xFF1C => self.volume.into_bits() | VOLUME_UNUSED_BITS,
            0xFF1D => WRITE_ONLY,
            0xFF1E => self.period_high.into_bits() | PERIOD_HIGH_UNUSED_BITS,
            0xFF30..=0xFF3F => self.read_wave_ram(address),
            _ => WRITE_ONLY,
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF1A => self.write_dac_enabled(value),
            0xFF1B => self.write_length(value),
            0xFF1C => self.volume = WaveVolume::from_bits(value),
            0xFF1D => self.period_low = value,
            0xFF1E => self.write_period_high(value),
            0xFF30..=0xFF3F => self.write_wave_ram(address, value),
            _ => {}
        }
    }
}

impl WaveChannel {
    pub fn new(mode: GbMode) -> Self {
        WaveChannel {
            enabled: false,
            dac_control: DacControl::from_bits(0),
            length: Length::new(WAVE_MAX_LENGTH),
            period: Period::new(),
            wave_position: 0,
            wave_ram: vec![0; WAVE_RAM_SIZE],
            volume: WaveVolume::from_bits(0),
            period_low: 0,
            period_high: PeriodHigh::from_bits(0),
            frame_sequencer_step: 0,
            mode,
        }
    }

    pub fn reset(&mut self, clear_length: bool) {
        self.enabled = false;
        self.dac_control = DacControl::from_bits(0);
        self.period = Period::new();
        self.wave_position = 0;
        self.volume = WaveVolume::from_bits(0);
        self.period_low = 0;
        self.period_high = PeriodHigh::from_bits(0);

        match clear_length {
            true => self.length = Length::new(WAVE_MAX_LENGTH),
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
        self.wave_position = ((self.wave_position as usize + steps) % WAVE_SAMPLES as usize) as u8;
    }

    pub fn cycle_length(&mut self) {
        if self.period_high.length_enable() {
            self.length.cycle();
            if self.length.expired() {
                self.enabled = false;
            }
        }
    }

    pub fn digital_output(&self) -> u8 {
        if !self.enabled {
            return 0;
        }

        let byte = self.wave_ram[(self.wave_position / 2) as usize];
        let sample = match self.wave_position.is_multiple_of(2) {
            true => byte >> 4,
            false => byte & 0x0F,
        };

        match self.volume.level() {
            VolumeLevel::Mute => 0,
            VolumeLevel::Full => sample,
            VolumeLevel::Half => sample >> 1,
            VolumeLevel::Quarter => sample >> 2,
        }
    }

    fn trigger(&mut self) {
        if self.enabled && self.period.timer() <= TICKS_PER_PERIOD_STEP && self.mode != GbMode::Color {
            self.corrupt_wave_ram();
        }

        self.wave_position = 0;
        if self.dac_control.enabled() {
            self.enabled = true;
        }

        self.period.trigger(self.period_reload());
        self.period.delay_wave_trigger();
        self.length.reload();
    }

    fn write_dac_enabled(&mut self, value: u8) {
        self.dac_control = DacControl::from_bits(value);
        if !self.dac_control.enabled() {
            self.enabled = false;
        }
    }

    fn write_length(&mut self, value: u8) {
        self.length.set_initial_time(value);
        self.length.initialize();
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

    fn wave_ram_accessible(&self) -> bool {
        !self.enabled || self.period.reloaded() || self.mode == GbMode::Color
    }

    fn wave_ram_index(&self, address: u16) -> usize {
        match self.enabled {
            true => (self.wave_position / 2) as usize,
            false => (address & 0x0F) as usize,
        }
    }

    fn read_wave_ram(&self, address: u16) -> u8 {
        match self.wave_ram_accessible() {
            true => self.wave_ram[self.wave_ram_index(address)],
            false => WRITE_ONLY,
        }
    }

    fn write_wave_ram(&mut self, address: u16, value: u8) {
        if self.wave_ram_accessible() {
            let index = self.wave_ram_index(address);
            self.wave_ram[index] = value;
        }
    }

    fn corrupt_wave_ram(&mut self) {
        let position = (self.wave_position.div_ceil(2) % WAVE_RAM_SIZE as u8) as usize;
        match position < 4 {
            true => self.wave_ram[0] = self.wave_ram[position],
            false => {
                let aligned = position & !0b11;
                for offset in 0..4 {
                    self.wave_ram[offset] = self.wave_ram[aligned + offset];
                }
            }
        }
    }
}
