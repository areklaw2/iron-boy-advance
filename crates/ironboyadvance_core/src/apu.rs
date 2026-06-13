use std::{cell::RefCell, rc::Rc};

use getset::Getters;
use ironboyadvance_arm7tdmi::CPU_CLOCK_SPEED;
use ironboyadvance_common::{memory::SystemMemoryAccess, register_ops::RegisterOps, scheduler::Scheduler};
use tracing::debug;

use crate::{
    apu::{
        noise::NoiseChannel,
        pulse::PulseChannel,
        sound::{DmaSoundControl, PsgSoundControl, PsgVolumeRatio, SoundBias, SoundStatus},
        wave::WaveChannel,
    },
    events::{ApuEvent, GbaEvent},
};

pub const APU_SAMPLING_FREQUENCY: usize = 32768; // Hz
const SAMPLE_CYCLES: usize = CPU_CLOCK_SPEED as usize / APU_SAMPLING_FREQUENCY;
const FRAME_SEQUENCER_FREQUENCY: usize = 512; // Hz
const FRAME_SEQUENCER_CYCLES: usize = CPU_CLOCK_SPEED as usize / FRAME_SEQUENCER_FREQUENCY;

mod length;
mod noise;
mod period;
mod pulse;
mod sound;
mod sweep;
mod volume_envelope;
mod wave;

#[derive(Getters)]
pub struct Apu {
    ch1: PulseChannel,
    ch2: PulseChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,
    psg_sound_control: PsgSoundControl,
    dma_sound_control: DmaSoundControl,
    sound_status: SoundStatus,
    sound_bias: SoundBias,
    #[getset(get = "pub")]
    audio_buffer: Vec<(f32, f32)>,
    frame_sequencer_step: usize,
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
}

impl SystemMemoryAccess for Apu {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            0x04000060..=0x04000067 => self.ch1.read_8(address),
            0x04000068..=0x0400006F => self.ch2.read_8(address),
            0x04000070..=0x04000077 => self.ch3.read_8(address),
            0x04000078..=0x0400007F => self.ch4.read_8(address),
            0x04000080..=0x04000081 => self.psg_sound_control.read_byte(address),
            0x04000082..=0x04000083 => self.dma_sound_control.read_byte(address),
            0x04000084..=0x04000087 => self.read_sound_status(address),
            0x04000088..=0x0400008B => self.sound_bias.read_byte(address),
            0x04000090..=0x0400009F => self.ch3.read_8(address),
            _ => {
                debug!("Invalid byte read for Apu Register: {:#010X}", address);
                0
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        if !self.sound_status.master_enable() && (0x04000060..=0x04000081).contains(&address) {
            return;
        }

        match address {
            0x04000060..=0x04000067 => self.ch1.write_8(address, value),
            0x04000068..=0x0400006F => self.ch2.write_8(address, value),
            0x04000070..=0x04000077 => self.ch3.write_8(address, value),
            0x04000078..=0x0400007F => self.ch4.write_8(address, value),
            0x04000080..=0x04000081 => self.psg_sound_control.write_byte(address, value),
            0x04000082..=0x04000083 => self.dma_sound_control.write_byte(address, value),
            0x04000084..=0x04000087 => self.write_sound_status(address, value),
            0x04000088..=0x0400008B => self.sound_bias.write_byte(address, value),
            0x04000090..=0x0400009F => self.ch3.write_8(address, value),
            _ => debug!("Invalid byte write for Apu Register: {:#010X}", address),
        }
    }
}

impl Apu {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbaEvent>>>) -> Self {
        scheduler
            .borrow_mut()
            .schedule((GbaEvent::Apu(ApuEvent::Sample), SAMPLE_CYCLES));
        scheduler
            .borrow_mut()
            .schedule((GbaEvent::Apu(ApuEvent::FrameSequence), FRAME_SEQUENCER_CYCLES));

        Self {
            ch1: PulseChannel::new(true),
            ch2: PulseChannel::new(false),
            ch3: WaveChannel::new(),
            ch4: NoiseChannel::new(),
            psg_sound_control: PsgSoundControl::from_bits(0),
            dma_sound_control: DmaSoundControl::from_bits(0),
            sound_status: SoundStatus::from_bits(0),
            sound_bias: SoundBias::from_bits(0x200),
            audio_buffer: Vec::new(),
            frame_sequencer_step: 0,
            scheduler,
        }
    }

    pub fn handle_event(&mut self, event: ApuEvent) {
        match event {
            ApuEvent::Sample => self.handle_sample(),
            ApuEvent::FrameSequence => self.handle_frame_sequence(),
        }
    }

    pub fn clear_audio_buffer(&mut self) {
        self.audio_buffer.clear();
    }

    fn handle_sample(&mut self) {
        self.ch1.cycle(SAMPLE_CYCLES);
        self.ch2.cycle(SAMPLE_CYCLES);
        self.ch3.cycle(SAMPLE_CYCLES);
        self.ch4.cycle(SAMPLE_CYCLES);

        let sample = match self.sound_status.master_enable() {
            true => self.mix(),
            false => (0.0, 0.0),
        };
        self.audio_buffer.push(sample);

        self.scheduler
            .borrow_mut()
            .schedule((GbaEvent::Apu(ApuEvent::Sample), SAMPLE_CYCLES));
    }

    fn mix(&self) -> (f32, f32) {
        let channels = [
            self.ch1.dac_output(),
            self.ch2.dac_output(),
            self.ch3.dac_output(),
            self.ch4.dac_output(),
        ];
        let control = &self.psg_sound_control;

        let left_enable = [
            control.ch1_left_enable(),
            control.ch2_left_enable(),
            control.ch3_left_enable(),
            control.ch4_left_enable(),
        ];
        let right_enable = [
            control.ch1_right_enable(),
            control.ch2_right_enable(),
            control.ch3_right_enable(),
            control.ch4_right_enable(),
        ];

        let mut left = 0.0;
        let mut right = 0.0;
        for i in 0..4 {
            if left_enable[i] {
                left += channels[i];
            }
            if right_enable[i] {
                right += channels[i];
            }
        }

        left *= (control.left_volume() as f32 + 1.0) / 8.0;
        right *= (control.right_volume() as f32 + 1.0) / 8.0;

        let ratio = match self.dma_sound_control.psg_volume_ratio() {
            PsgVolumeRatio::Ratio25 => 0.25,
            PsgVolumeRatio::Ratio50 => 0.50,
            _ => 1.0,
        };
        (left * ratio / 4.0, right * ratio / 4.0)
    }

    fn handle_frame_sequence(&mut self) {
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

        self.frame_sequencer_step = (step + 1) % 8;
        self.scheduler
            .borrow_mut()
            .schedule((GbaEvent::Apu(ApuEvent::FrameSequence), FRAME_SEQUENCER_CYCLES));
    }

    fn read_sound_status(&self, address: u32) -> u8 {
        let mut status = SoundStatus::from_bits(self.sound_status.register());
        status.set_ch1_on(self.ch1.enabled());
        status.set_ch2_on(self.ch2.enabled());
        status.set_ch3_on(self.ch3.enabled());
        status.set_ch4_on(self.ch4.enabled());
        status.read_byte(address)
    }

    fn write_sound_status(&mut self, address: u32, value: u8) {
        let was_enabled = self.sound_status.master_enable();
        self.sound_status.write_byte(address, value);
        if was_enabled && !self.sound_status.master_enable() {
            self.ch1.reset();
            self.ch2.reset();
            self.ch3.reset();
            self.ch4.reset();
            self.psg_sound_control = PsgSoundControl::from_bits(0);
        }
    }
}
