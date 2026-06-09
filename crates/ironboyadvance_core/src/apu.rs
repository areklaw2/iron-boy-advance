use std::{cell::RefCell, rc::Rc};

use getset::Getters;
use ironboyadvance_arm7tdmi::CPU_CLOCK_SPEED;
use ironboyadvance_common::{memory::SystemMemoryAccess, register_ops::RegisterOps, scheduler::Scheduler};
use tracing::debug;

use crate::{
    apu::{
        pulse::PulseChannel,
        sound::{DmaSoundControl, PsgSoundControl, PsgVolumeRatio, SoundBias, SoundStatus},
    },
    events::{ApuEvent, GbaEvent},
};

pub const APU_SAMPLING_FREQUENCY: usize = 32768; // Hz
const SAMPLE_CYCLES: usize = CPU_CLOCK_SPEED as usize / APU_SAMPLING_FREQUENCY;
const FRAME_SEQUENCER_FREQUENCY: usize = 512; // Hz
const FRAME_SEQUENCER_CYCLES: usize = CPU_CLOCK_SPEED as usize / FRAME_SEQUENCER_FREQUENCY;

mod length;
mod period;
mod pulse;
mod sound;
mod sweep;
mod volume_envelope;

#[derive(Getters)]
pub struct Apu {
    ch1: PulseChannel,
    ch2: PulseChannel,
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
            0x04000080..=0x04000081 => self.psg_sound_control.read_byte(address),
            0x04000082..=0x04000083 => self.dma_sound_control.read_byte(address),
            0x04000084..=0x04000087 => self.sound_status.read_byte(address),
            0x04000088..=0x0400008B => self.sound_bias.read_byte(address),
            _ => {
                debug!("Invalid byte read for Apu Register: {:#010X}", address);
                0
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            0x04000060..=0x04000067 => self.ch1.write_8(address, value),
            0x04000068..=0x0400006F => self.ch2.write_8(address, value),
            0x04000080..=0x04000081 => self.psg_sound_control.write_byte(address, value),
            0x04000082..=0x04000083 => self.dma_sound_control.write_byte(address, value),
            0x04000084..=0x04000087 => self.sound_status.write_byte(address, value),
            0x04000088..=0x0400008B => self.sound_bias.write_byte(address, value),
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
        let channels = [self.ch1.dac_output(), self.ch2.dac_output(), 0.0, 0.0];
        let control = &self.psg_sound_control;

        let left_enable = [control.ch1_left_enable(), control.ch2_left_enable(), false, false];
        let right_enable = [control.ch1_right_enable(), control.ch2_right_enable(), false, false];

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
        }

        if matches!(step, 0 | 2 | 4 | 6) {
            self.ch1.cycle_length();
            self.ch2.cycle_length();
        }

        if matches!(step, 2 | 6) {
            self.ch1.cycle_sweep();
        }

        self.frame_sequencer_step = (step + 1) % 8;
        self.scheduler
            .borrow_mut()
            .schedule((GbaEvent::Apu(ApuEvent::FrameSequence), FRAME_SEQUENCER_CYCLES));
    }
}
