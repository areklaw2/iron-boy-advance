use std::{cell::RefCell, rc::Rc};

use getset::Getters;
use ironboyadvance_arm7tdmi::CPU_CLOCK_SPEED;
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use tracing::debug;

use crate::events::{ApuEvent, GbaEvent};

const SAMPLING_FREQUENCY: usize = 32768; // Hz
const SAMPLE_CYCLES: usize = CPU_CLOCK_SPEED as usize / SAMPLING_FREQUENCY;
const FRAME_SEQUENCER_FREQUENCY: usize = 512; // Hz
const FRAME_SEQUENCER_CYCLES: usize = CPU_CLOCK_SPEED as usize / FRAME_SEQUENCER_FREQUENCY;

#[derive(Getters)]
pub struct Apu {
    #[getset(get = "pub")]
    audio_buffer: Vec<(f32, f32)>,
    frame_sequencer_step: usize,
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
}

impl SystemMemoryAccess for Apu {
    fn read_8(&self, address: u32) -> u8 {
        debug!("Read byte not implemented for APU register: {:#010X}", address);
        0
    }

    fn write_8(&mut self, address: u32, value: u8) {
        debug!(
            "Write byte not implemented for APU register: {:#010X}, value: {:#04X}",
            address, value
        );
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
