use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ironboyadvance_arm7tdmi::{CPU_CLOCK_SPEED, cpu::Arm7tdmiCpu};
use ironboyadvance_common::scheduler::Scheduler;
use thiserror::Error;

use crate::{
    bios::{Bios, BiosError},
    cartridge::{Cartridge, CartridgeError},
    events::{GbaEvent, InterruptEvent},
    system_bus::SystemBus,
    system_control::HaltMode,
};

mod apu;
mod bios;
mod cartridge;
mod dma_control;
mod events;
mod interrupt_control;
mod io_registers;
mod keypad;
mod memory;
mod ppu;
mod system_bus;
mod system_control;
mod timer_control;

pub const FPS: f32 = CPU_CLOCK_SPEED as f32 / CYCLES_PER_FRAME as f32;

pub use apu::APU_SAMPLING_FREQUENCY;
pub use keypad::KeypadButton;
pub use ppu::{CYCLES_PER_FRAME, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};

#[derive(Error, Debug)]
pub enum GbaError {
    #[error("Failed to load bios: {0}")]
    BiosError(#[from] BiosError),
    #[error("Failed to load cartridge: {0}")]
    CartridgeError(#[from] CartridgeError),
}

pub struct GameBoyAdvance {
    arm7tdmi: Arm7tdmiCpu<SystemBus>,
    // may end up making a common cpu trait
    // sharp_sm83: SharpSm83Cpu<SystemBus>,
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
}

impl GameBoyAdvance {
    pub fn new(
        rom_path: PathBuf,
        rom_buffer: Vec<u8>,
        bios_buffer: Vec<u8>,
        show_logs: bool,
    ) -> Result<GameBoyAdvance, GbaError> {
        let scheduler = Rc::new(RefCell::new(Scheduler::new()));
        let cartridge = Cartridge::load(rom_path, rom_buffer)?;
        let bios = Bios::load(bios_buffer)?;
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
                } else if self.arm7tdmi.bus().dma_active() {
                    self.arm7tdmi.bus_mut().run_dma();
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
            .schedule_at_timestamp(GbaEvent::FrameComplete, end_time);

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
        loop {
            let Some((event, timestamp)) = self.scheduler.borrow_mut().pop() else {
                return false;
            };

            match event {
                GbaEvent::FrameComplete => return true,
                GbaEvent::Interrupt(interrupt_event) => self.arm7tdmi.bus_mut().raise_interrupt(interrupt_event),
                GbaEvent::Timer(timers_event) => self.arm7tdmi.bus_mut().handle_timer_event(timers_event),
                GbaEvent::Ppu(ppu_event) => self.arm7tdmi.bus_mut().handle_ppu_event(ppu_event, timestamp),
                GbaEvent::Apu(apu_event) => self.arm7tdmi.bus_mut().handle_apu_event(apu_event),
                GbaEvent::Dma(dma_event) => self.arm7tdmi.bus_mut().handle_dma_event(dma_event),
            }
        }
    }

    pub fn frame_buffer(&self) -> &[u32] {
        self.arm7tdmi.bus().io_registers().ppu().frame_buffer()
    }

    pub fn audio_buffer(&self) -> &[(f32, f32)] {
        self.arm7tdmi.bus().io_registers().apu().audio_buffer()
    }

    pub fn clear_audio_buffer(&mut self) {
        self.arm7tdmi.bus_mut().io_registers_mut().apu_mut().clear_audio_buffer();
    }

    pub fn handle_pressed_buttons(&mut self, input: u16) {
        let keypad = self.arm7tdmi.bus_mut().io_registers_mut().keypad_mut();
        keypad.set_key_input(input);
        if keypad.keypad_interrupt_raised() {
            self.arm7tdmi.bus_mut().raise_interrupt(InterruptEvent::Keypad);
        }
    }
}
