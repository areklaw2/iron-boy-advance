use std::{cell::RefCell, rc::Rc};

use getset::Getters;
use ironboyadvance_arm7tdmi::CPU_CLOCK_SPEED;
use ironboyadvance_common::{memory::SystemMemoryAccess, register_ops::RegisterOps, scheduler::Scheduler};
use tracing::debug;

use crate::{
    apu::sound::{DmaSoundControl, PsgSoundControl, SoundBias, SoundStatus},
    events::{ApuEvent, GbaEvent},
};

const SAMPLING_FREQUENCY: usize = 32768; // Hz
const SAMPLE_CYCLES: usize = CPU_CLOCK_SPEED as usize / SAMPLING_FREQUENCY;
const FRAME_SEQUENCER_FREQUENCY: usize = 512; // Hz
const FRAME_SEQUENCER_CYCLES: usize = CPU_CLOCK_SPEED as usize / FRAME_SEQUENCER_FREQUENCY;

mod sound;

#[derive(Getters)]
pub struct Apu {
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

    fn handle_sample(&mut self) {
        self.audio_buffer.push((0.0, 0.0));
        self.scheduler
            .borrow_mut()
            .schedule((GbaEvent::Apu(ApuEvent::Sample), SAMPLE_CYCLES));
    }

    fn handle_frame_sequence(&mut self) {
        self.frame_sequencer_step = (self.frame_sequencer_step + 1) % 8;
        self.scheduler
            .borrow_mut()
            .schedule((GbaEvent::Apu(ApuEvent::FrameSequence), FRAME_SEQUENCER_CYCLES));
    }
}
