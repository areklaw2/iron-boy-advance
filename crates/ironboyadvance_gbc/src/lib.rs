use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ironboyadvance_common::{emulator::Emulator, scheduler::Scheduler};
use ironboyadvance_sm83::{GbMode, cpu::Sm83, memory::MemoryInterface};
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

pub const VIEWPORT_WIDTH: usize = 160;
pub const VIEWPORT_HEIGHT: usize = 144;
pub const CPU_CLOCK_SPEED: usize = 4_194_304;
pub const CYCLES_PER_FRAME: usize = 70_224;
pub const FPS: f32 = CPU_CLOCK_SPEED as f32 / CYCLES_PER_FRAME as f32;
pub const SAMPLE_RATE: u32 = 32768;

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
        rom_path: PathBuf,
        rom_buffer: Vec<u8>,
        boot_rom_buffer: Vec<u8>,
        show_logs: bool,
    ) -> Result<GameBoyColor, GbcError> {
        let scheduler = Rc::new(RefCell::new(Scheduler::new()));
        let cartridge = Cartridge::load(rom_path, rom_buffer)?;
        let boot_rom = BootRom::load(boot_rom_buffer)?;
        let skip_boot = !boot_rom.loaded();
        let mode = cartridge.mode();

        let gbc = GameBoyColor {
            sm83: Sm83::new(
                SystemBus::new(cartridge, boot_rom, scheduler.clone()),
                show_logs,
                skip_boot,
                mode,
            ),
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
                false => self.sm83.bus_mut().idle_cycle(),
            },
            false => {
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
        if joypad.set_button_input(input) {
            self.sm83.bus_mut().raise_interrupt(InterruptEvent::Joypad);
        }
    }
}
