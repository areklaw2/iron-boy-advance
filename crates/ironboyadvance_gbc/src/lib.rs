use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ironboyadvance_common::{
    emulator::{Emulator, System, SystemInspection},
    memory::SystemMemoryAccess,
    scheduler::Scheduler,
};
use ironboyadvance_sm83::{CPU_CLOCK_SPEED, GbMode, HaltMode, cpu::Sm83, memory::MemoryInterface};
use thiserror::Error;

use crate::{
    boot_rom::{BootRom, BootRomError},
    cartridge::{Cartridge, CartridgeError},
    events::{GbcEvent, InterruptEvent},
    system_bus::SystemBus,
};

mod apu;
mod boot_rom;
mod cartridge;
mod dma_control;
mod events;
mod interrupt_control;
mod io_registers;
mod joypad;
mod memory;
mod ppu;
mod serial_transfer;
mod speed_control;
mod system_bus;
mod timer;

pub const FPS: f32 = CPU_CLOCK_SPEED as f32 / CYCLES_PER_FRAME as f32;

pub use apu::SAMPLE_RATE;

pub use ppu::{CYCLES_PER_FRAME, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};

#[derive(Error, Debug)]
pub enum GbcError {
    #[error("Failed to load boot rom: {0}")]
    BootRomError(#[from] BootRomError),
    #[error("Failed to load cartridge: {0}")]
    CartridgeError(#[from] CartridgeError),
}

pub struct GameBoyColor {
    sm83: Sm83<SystemBus>,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
}

impl GameBoyColor {
    pub fn new(
        kind: System,
        rom_path: PathBuf,
        rom_buffer: Vec<u8>,
        boot_rom_buffer: Vec<u8>,
        show_logs: bool,
    ) -> Result<GameBoyColor, GbcError> {
        let scheduler = Rc::new(RefCell::new(Scheduler::new()));
        let cartridge = Cartridge::load(rom_path, rom_buffer)?;
        let boot_rom = BootRom::load(boot_rom_buffer)?;
        let skip_boot = !boot_rom.loaded();
        let mode = match kind {
            System::Gb => GbMode::ColorAsMonochrome,
            _ => cartridge.mode(),
        };

        let gbc = GameBoyColor {
            sm83: Sm83::new(
                SystemBus::new(cartridge, boot_rom, mode, scheduler.clone()),
                show_logs,
                skip_boot,
                mode,
            ),
            scheduler,
        };
        Ok(gbc)
    }

    pub fn cycle(&mut self) {
        match self.sm83.halt_mode() {
            HaltMode::Stopped => self.sm83.bus_mut().idle_cycle(),
            HaltMode::Halted => match self.sm83.bus().interrupts_pending() {
                true => {
                    self.sm83.set_halt_mode(HaltMode::Running);
                    self.sm83.irq();
                }
                false => self.sm83.bus_mut().idle_cycle(),
            },
            HaltMode::Running => {
                if self.sm83.bus().interrupts_pending() {
                    self.sm83.irq();
                }
                self.sm83.cycle();
            }
        }
    }
}

impl Emulator for GameBoyColor {
    fn run(&mut self, cycles: usize, overshoot: usize) -> usize {
        let start_time = self.scheduler.borrow().timestamp();
        let end_time = start_time + cycles - overshoot;

        while self.scheduler.borrow().timestamp() < end_time {
            self.cycle();
        }

        let elapsed = self.scheduler.borrow().timestamp() - start_time;
        let target = cycles - overshoot;
        elapsed.saturating_sub(target)
    }

    fn frame_buffer(&self) -> &[u32] {
        self.sm83.bus().io_registers().ppu().frame_buffer()
    }

    fn audio_buffer(&self) -> &[(f32, f32)] {
        self.sm83.bus().io_registers().apu().audio_buffer()
    }

    fn clear_audio_buffer(&mut self) {
        self.sm83.bus_mut().io_registers_mut().apu_mut().clear_audio_buffer();
    }

    fn handle_pressed_buttons(&mut self, input: u16) {
        let joypad = self.sm83.bus_mut().io_registers_mut().joypad_mut();
        let interrupt_requested = joypad.set_button_input(input);
        let selected_pressed = joypad.selected_pressed();

        if interrupt_requested {
            self.sm83.bus_mut().raise_interrupt(InterruptEvent::Joypad);
        }

        if selected_pressed && self.sm83.halt_mode() == HaltMode::Stopped {
            self.sm83.set_halt_mode(HaltMode::Running);
        }
    }
}

impl SystemInspection for GameBoyColor {
    fn serial_output(&self) -> &[u8] {
        self.sm83.bus().io_registers().serial_transfer().output()
    }

    fn read_memory(&self, address: u32) -> u8 {
        self.sm83.bus().read_8(address as u16)
    }
}
