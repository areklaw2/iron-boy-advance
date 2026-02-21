use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ironboyadvance_arm7tdmi::{CPU_CLOCK_SPEED, cpu::Arm7tdmiCpu};
use thiserror::Error;

use crate::{
    bios::{Bios, BiosError},
    cartridge::{Cartridge, CartridgeError},
    ppu::CYCLES_PER_FRAME,
    scheduler::{
        Scheduler,
        event::{EventType, FutureEvent},
    },
    system_bus::SystemBus,
    system_control::HaltMode,
};

mod bios;
mod cartridge;
mod interrupt_control;
mod io_registers;
mod memory;
mod ppu;
mod scheduler;
mod system_bus;
mod system_control;

pub const FPS: f32 = CPU_CLOCK_SPEED as f32 / CYCLES_PER_FRAME as f32;

pub use ppu::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH};

#[derive(Error, Debug)]
pub enum GbaError {
    #[error("Failed to load cartridge: {0}")]
    CartridgeError(#[from] CartridgeError),
    #[error("Failed to load cartridge: {0}")]
    BiosError(#[from] BiosError),
    #[error("Path cannot be empty")]
    EmptyPath,
    #[error("Unable to extract filename")]
    InvalidRomPath,
}

pub struct GameBoyAdvance {
    arm7tdmi: Arm7tdmiCpu<SystemBus>,
    // may end up making a common cpu trait
    // sharp_sm83: SharpSm83Cpu<SystemBus>,
    scheduler: Rc<RefCell<Scheduler>>,
    rom_name: String,
}

impl GameBoyAdvance {
    pub fn new(rom_path: PathBuf, bios_path: Option<PathBuf>, show_logs: bool) -> Result<GameBoyAdvance, GbaError> {
        let rom_name = rom_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
            .ok_or(GbaError::InvalidRomPath)?;

        let scheduler = Rc::new(RefCell::new(Scheduler::new()));
        let cartridge = Cartridge::load(rom_path)?;
        let bios = Bios::load(bios_path)?;
        let skip_bios = !bios.loaded();
        let gba = GameBoyAdvance {
            arm7tdmi: Arm7tdmiCpu::new(SystemBus::new(cartridge, bios, scheduler.clone()), show_logs, skip_bios),
            scheduler,
            rom_name,
        };
        Ok(gba)
    }

    pub fn cycle(&mut self) {
        match self.arm7tdmi.bus().halt_mode() {
            HaltMode::Stopped => todo!(),
            HaltMode::Halted => {
                if self.arm7tdmi.bus().interrupt_pending() {
                    self.arm7tdmi.bus_mut().un_halt();
                    self.arm7tdmi.irq();
                } else {
                    self.scheduler.borrow_mut().step_to_next_event();
                }
            }
            HaltMode::Running => {
                if self.arm7tdmi.bus().interrupt_pending() {
                    self.arm7tdmi.irq();
                }
                self.arm7tdmi.cycle();
            }
        }
    }

    pub fn run(&mut self, overshoot: usize) -> usize {
        let start_time = self.scheduler.borrow().timestamp();
        let end_time = start_time + CYCLES_PER_FRAME - overshoot;

        self.scheduler
            .borrow_mut()
            .schedule_at_timestamp(EventType::FrameComplete, end_time);

        'events: loop {
            while self.scheduler.borrow().timestamp() <= self.scheduler.borrow().timestamp_of_next_event() {
                self.cycle();
            }

            if self.handle_events() {
                break 'events;
            }
        }

        self.scheduler.borrow().timestamp() - start_time
    }

    fn handle_events(&mut self) -> bool {
        let mut scheduler = self.scheduler.borrow_mut();
        while let Some((event, timestamp)) = scheduler.pop() {
            let future_events: Vec<FutureEvent> = match event {
                EventType::FrameComplete => return true,
                EventType::Interrupt(interrupt_event) => self.arm7tdmi.bus_mut().raise_interrupt(interrupt_event),
                EventType::Timer(_timer_event) => vec![],
                EventType::Ppu(ppu_event) => self.arm7tdmi.bus_mut().handle_ppu_event(ppu_event),
                EventType::Apu(_apu_event) => vec![],
            };

            for (event_type, time) in future_events {
                scheduler.schedule_at_timestamp(event_type, timestamp + time);
            }
        }
        false
    }

    pub fn frame_buffer(&self) -> &[u32] {
        self.arm7tdmi.bus().io_registers().ppu().frame_buffer()
    }
}
