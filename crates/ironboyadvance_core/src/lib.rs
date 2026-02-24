use std::{cell::RefCell, rc::Rc};

use getset::Getters;
use ironboyadvance_arm7tdmi::{CPU_CLOCK_SPEED, cpu::Arm7tdmiCpu};
use thiserror::Error;

use crate::{
    bios::Bios,
    cartridge::{Cartridge, CartridgeError},
    scheduler::{
        Scheduler,
        event::{EventType, FutureEvent, InterruptEvent},
    },
    system_bus::SystemBus,
    system_control::HaltMode,
};

mod bios;
mod cartridge;
mod interrupt_control;
mod io_registers;
mod keypad;
mod memory;
mod ppu;
mod scheduler;
mod system_bus;
mod system_control;

pub const FPS: f32 = CPU_CLOCK_SPEED as f32 / CYCLES_PER_FRAME as f32;

pub use keypad::KeypadButton;
pub use ppu::{CYCLES_PER_FRAME, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};

#[derive(Error, Debug)]
pub enum GbaError {
    #[error("Failed to load cartridge: {0}")]
    CartridgeError(#[from] CartridgeError),
}

#[derive(Getters)]
pub struct GameBoyAdvance {
    arm7tdmi: Arm7tdmiCpu<SystemBus>,
    // may end up making a common cpu trait
    // sharp_sm83: SharpSm83Cpu<SystemBus>,
    scheduler: Rc<RefCell<Scheduler>>,
}

impl GameBoyAdvance {
    pub fn new(rom_buffer: Vec<u8>, bios_buffer: Box<[u8]>, show_logs: bool) -> Result<GameBoyAdvance, GbaError> {
        let scheduler = Rc::new(RefCell::new(Scheduler::new()));
        let cartridge = Cartridge::load(rom_buffer)?;
        let bios = Bios::load(bios_buffer);
        let skip_bios = !bios.loaded();
        let gba = GameBoyAdvance {
            arm7tdmi: Arm7tdmiCpu::new(SystemBus::new(cartridge, bios, scheduler.clone()), show_logs, skip_bios),
            scheduler,
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

    pub fn run(&mut self, cycles: usize, overshoot: usize) -> usize {
        let start_time = self.scheduler.borrow().timestamp();
        let end_time = start_time + cycles - overshoot;

        self.scheduler
            .borrow_mut()
            .schedule_at_timestamp(EventType::FrameComplete, end_time);

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

    pub fn handle_pressed_buttons(&mut self, input: u16) {
        let keypad = self.arm7tdmi.bus_mut().io_registers_mut().keypad_mut();
        keypad.set_key_input(input);
        if keypad.keypad_interrupt_raised() {
            self.arm7tdmi.bus_mut().raise_interrupt(InterruptEvent::Keypad);
        }
    }
}
