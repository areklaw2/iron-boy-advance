use std::{cell::RefCell, rc::Rc};

use getset::Getters;
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use ironboyadvance_sm83::{CPU_CLOCK_SPEED, GbMode};

use crate::{
    apu::{
        high_pass_filter::HighPassFilter,
        noise::NoiseChannel,
        pulse::PulseChannel,
        sound::{MASTER_CONTROL_UNUSED_BITS, MasterControl, MasterVolume, SoundPanning},
        wave::WaveChannel,
    },
    events::{ApuEvent, GbcEvent},
};

mod high_pass_filter;
mod length;
mod noise;
mod period;
mod pulse;
mod sound;
mod sweep;
mod volume_envelope;
mod wave;

pub const SAMPLE_RATE: u32 = 32768;
const SAMPLE_CYCLES: usize = CPU_CLOCK_SPEED as usize / SAMPLE_RATE as usize;
const FRAME_SEQUENCER_STEPS: usize = 8;
const VOLUME_STEPS: f32 = 8.0;
const CHANNEL_COUNT: usize = 4;
const LENGTH_ONLY_MASK: u8 = 0x3F;

#[derive(Getters)]
pub struct Apu {
    ch1: PulseChannel,
    ch2: PulseChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,
    master_volume: MasterVolume,
    panning: SoundPanning,
    enabled: bool,
    frame_sequencer_step: usize,
    high_pass: HighPassFilter,
    #[getset(get = "pub")]
    audio_buffer: Vec<(f32, f32)>,
    mode: GbMode,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
}

impl Apu {
    pub fn new(mode: GbMode, scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        scheduler
            .borrow_mut()
            .schedule((GbcEvent::Apu(ApuEvent::Sample), SAMPLE_CYCLES));

        Apu {
            ch1: PulseChannel::new(true),
            ch2: PulseChannel::new(false),
            ch3: WaveChannel::new(mode),
            ch4: NoiseChannel::new(),
            master_volume: MasterVolume::from_bits(0),
            panning: SoundPanning::from_bits(0),
            enabled: false,
            frame_sequencer_step: 0,
            high_pass: HighPassFilter::new(),
            audio_buffer: Vec::new(),
            mode,
            scheduler,
        }
    }

    pub fn clear_audio_buffer(&mut self) {
        self.audio_buffer.clear();
    }

    pub fn cycle(&mut self, cycles: usize) {
        if !self.enabled {
            return;
        }

        self.ch1.cycle(cycles);
        self.ch2.cycle(cycles);
        self.ch3.cycle(cycles);
        self.ch4.cycle(cycles);
    }

    pub fn handle_event(&mut self, event: ApuEvent) {
        match event {
            ApuEvent::Sample => self.handle_sample(),
            ApuEvent::FrameSequence => self.handle_frame_sequence(),
        }
    }

    fn handle_sample(&mut self) {
        let (left, right) = match self.enabled {
            true => self.mix(),
            false => (0.0, 0.0),
        };
        let (left, right) = self.high_pass.process(left, right);
        self.audio_buffer.push((left, right));

        self.scheduler
            .borrow_mut()
            .schedule((GbcEvent::Apu(ApuEvent::Sample), SAMPLE_CYCLES));
    }

    fn mix(&self) -> (f32, f32) {
        let channels = [
            self.ch1.digital_output(),
            self.ch2.digital_output(),
            self.ch3.digital_output(),
            self.ch4.digital_output(),
        ];

        let mut left = 0.0;
        let mut right = 0.0;
        for (channel, digital) in channels.into_iter().enumerate() {
            let sample = (digital as f32 / 7.5) - 1.0;
            if self.panning.left_enabled(channel) {
                left += sample;
            }
            if self.panning.right_enabled(channel) {
                right += sample;
            }
        }

        left *= (self.master_volume.left_volume() as f32 + 1.0) / VOLUME_STEPS;
        right *= (self.master_volume.right_volume() as f32 + 1.0) / VOLUME_STEPS;

        let scale = CHANNEL_COUNT as f32;
        ((left / scale).clamp(-1.0, 1.0), (right / scale).clamp(-1.0, 1.0))
    }

    fn handle_frame_sequence(&mut self) {
        if !self.enabled {
            return;
        }

        let step = self.frame_sequencer_step;

        if step == 7 {
            self.ch1.cycle_envelope();
            self.ch2.cycle_envelope();
            self.ch4.cycle_envelope();
        }

        if matches!(step, 0 | 2 | 4 | 6) {
            self.ch1.cycle_length();
            self.ch2.cycle_length();
            self.ch3.cycle_length();
            self.ch4.cycle_length();
        }

        if matches!(step, 2 | 6) {
            self.ch1.cycle_sweep();
        }

        self.set_frame_sequencer_step((step + 1) % FRAME_SEQUENCER_STEPS);
    }

    fn set_frame_sequencer_step(&mut self, step: usize) {
        self.frame_sequencer_step = step;
        self.ch1.set_frame_sequencer_step(step);
        self.ch2.set_frame_sequencer_step(step);
        self.ch3.set_frame_sequencer_step(step);
        self.ch4.set_frame_sequencer_step(step);
    }

    fn read_master_control(&self) -> u8 {
        let mut control = MasterControl::from_bits(0);
        control.set_enabled(self.enabled);
        control.set_ch1_on(self.ch1.enabled());
        control.set_ch2_on(self.ch2.enabled());
        control.set_ch3_on(self.ch3.enabled());
        control.set_ch4_on(self.ch4.enabled());
        control.into_bits() | MASTER_CONTROL_UNUSED_BITS
    }

    fn write_master_control(&mut self, value: u8) {
        let was_enabled = self.enabled;
        self.enabled = MasterControl::from_bits(value).enabled();

        if was_enabled && !self.enabled {
            self.power_off();
        }
    }

    fn power_off(&mut self) {
        let clear_length = self.mode == GbMode::Color;
        self.ch1.reset(clear_length);
        self.ch2.reset(clear_length);
        self.ch3.reset(clear_length);
        self.ch4.reset(clear_length);
        self.master_volume = MasterVolume::from_bits(0);
        self.panning = SoundPanning::from_bits(0);
        self.set_frame_sequencer_step(0);
    }

    fn length_write_allowed(&self) -> bool {
        self.enabled || self.mode != GbMode::Color
    }
}

impl SystemMemoryAccess for Apu {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF10..=0xFF14 => self.ch1.read_8(address),
            0xFF16..=0xFF19 => self.ch2.read_8(address),
            0xFF1A..=0xFF1E => self.ch3.read_8(address),
            0xFF20..=0xFF23 => self.ch4.read_8(address),
            0xFF24 => self.master_volume.into_bits(),
            0xFF25 => self.panning.into_bits(),
            0xFF26 => self.read_master_control(),
            0xFF30..=0xFF3F => self.ch3.read_8(address),
            _ => 0xFF,
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF26 => self.write_master_control(value),
            0xFF30..=0xFF3F => self.ch3.write_8(address, value),
            0xFF11 | 0xFF16 if self.length_write_allowed() => {
                let masked = match self.enabled {
                    true => value,
                    false => value & LENGTH_ONLY_MASK,
                };
                match address {
                    0xFF11 => self.ch1.write_8(address, masked),
                    _ => self.ch2.write_8(address, masked),
                }
            }
            0xFF1B if self.length_write_allowed() => self.ch3.write_8(address, value),
            0xFF20 if self.length_write_allowed() => self.ch4.write_8(address, value),
            _ if !self.enabled => {}
            0xFF10 | 0xFF12..=0xFF14 => self.ch1.write_8(address, value),
            0xFF17..=0xFF19 => self.ch2.write_8(address, value),
            0xFF1A | 0xFF1C..=0xFF1E => self.ch3.write_8(address, value),
            0xFF21..=0xFF23 => self.ch4.write_8(address, value),
            0xFF24 => self.master_volume = MasterVolume::from_bits(value),
            0xFF25 => self.panning = SoundPanning::from_bits(value),
            _ => {}
        }
    }
}
