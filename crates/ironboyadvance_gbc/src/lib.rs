use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ironboyadvance_common::{emulator::Emulator, scheduler::Scheduler};
use ironboyadvance_sm83::{GbMode, cpu::Sm83};
use thiserror::Error;

use crate::{events::GbcEvent, system_bus::SystemBus};

mod events;
mod interrupt_control;
mod io_registers;
mod memory;
mod system_bus;

pub const VIEWPORT_WIDTH: usize = 160;
pub const VIEWPORT_HEIGHT: usize = 144;
pub const CPU_CLOCK_SPEED: usize = 4_194_304;
pub const CYCLES_PER_FRAME: usize = 70_224;
pub const FPS: f32 = CPU_CLOCK_SPEED as f32 / CYCLES_PER_FRAME as f32;
pub const SAMPLE_RATE: u32 = 32768;

#[derive(Error, Debug)]
pub enum GbcError {}

pub struct GameBoyColor {
    sm83: Sm83<SystemBus>,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
}

impl GameBoyColor {
    pub fn new(rom_path: PathBuf, rom_buffer: Vec<u8>, show_logs: bool) -> Result<GameBoyColor, GbcError> {
        let skip_boot = true;
        let mode = GbMode::Color;
        let scheduler = Rc::new(RefCell::new(Scheduler::new()));
        let gbc = GameBoyColor {
            sm83: Sm83::new(SystemBus::new(), show_logs, skip_boot, mode),
            scheduler,
        };
        Ok(gbc)
    }

    pub fn cycle(&mut self) {
        match self.sm83.halted() {
            true => match self.sm83.bus().interrupts_pending() {
                true => {
                    self.sm83.un_halt();
                    self.sm83.irq();
                }
                false => self.scheduler.borrow_mut().step_to_next_event(),
            },
            false => {
                if self.sm83.bus().interrupts_pending() {
                    self.sm83.irq();
                }
                self.sm83.cycle();
            }
        }
    }

    fn handle_events(&mut self) -> bool {
        loop {
            let Some((event, _timestamp)) = self.scheduler.borrow_mut().pop() else {
                return false;
            };

            match event {
                GbcEvent::FrameComplete => return true,
                GbcEvent::Interrupt(interrupt_event) => self.sm83.bus_mut().raise_interrupt(interrupt_event),
                GbcEvent::Ppu(_) => todo!(),
                GbcEvent::Apu(_) => todo!(),
                GbcEvent::Timer(_) => todo!(),
                GbcEvent::Dma(_) => todo!(),
            }
        }
    }
}

impl Emulator for GameBoyColor {
    fn run(&mut self, cycles: usize, overshoot: usize) -> usize {
        let start_time = self.scheduler.borrow().timestamp();
        let end_time = start_time + cycles - overshoot;

        self.scheduler
            .borrow_mut()
            .schedule_at_timestamp(GbcEvent::FrameComplete, end_time);

        'events: loop {
            while self.scheduler.borrow().timestamp() < self.scheduler.borrow().timestamp_of_next_event() {
                self.cycle();
            }

            if self.handle_events() {
                break 'events;
            }
        }

        let elapsed = self.scheduler.borrow().timestamp() - start_time;
        let target = cycles - overshoot;
        elapsed.saturating_sub(target)
    }

    fn frame_buffer(&self) -> &[u32] {
        todo!()
    }

    fn audio_buffer(&self) -> &[(f32, f32)] {
        todo!()
    }

    fn clear_audio_buffer(&mut self) {
        todo!()
    }

    fn handle_pressed_buttons(&mut self, input: u16) {
        todo!()
    }
}
