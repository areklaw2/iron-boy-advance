use std::path::PathBuf;

use ironboyadvance_arm7tdmi_interpreter::Interpreter;
use ironboyadvance_gba::{GameBoyAdvance, GbaError};
use ironboyadvance_gbc::{GameBoyColor, GbcError};
use thiserror::Error;

pub use ironboyadvance_common::keypad::KeypadButton;
pub use ironboyadvance_common::system::{Console, System, Viewport, detect_console};

#[derive(Error, Debug)]
pub enum BootError {
    #[error("failed to initialize GBA: {0}")]
    Gba(#[from] GbaError),
    #[error("failed to initialize GBC: {0}")]
    Gbc(#[from] GbcError),
    #[error("unrecognized rom format")]
    UnknownFormat,
}

pub const GBA_VIEWPORT: Viewport = Viewport {
    width: ironboyadvance_gba::VIEWPORT_WIDTH,
    height: ironboyadvance_gba::VIEWPORT_HEIGHT,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemInfo {
    pub viewport: Viewport,
    pub fps: f32,
    pub sample_rate: u32,
    pub cycles_per_frame: usize,
}

pub fn system_info(kind: Console) -> SystemInfo {
    match kind {
        Console::Gba => SystemInfo {
            viewport: GBA_VIEWPORT,
            fps: ironboyadvance_gba::FPS,
            sample_rate: ironboyadvance_gba::APU_SAMPLING_FREQUENCY as u32,
            cycles_per_frame: ironboyadvance_gba::CYCLES_PER_FRAME,
        },
        Console::Gb | Console::Gbc => SystemInfo {
            viewport: Viewport {
                width: ironboyadvance_gbc::VIEWPORT_WIDTH,
                height: ironboyadvance_gbc::VIEWPORT_HEIGHT,
            },
            fps: ironboyadvance_gbc::FPS,
            sample_rate: ironboyadvance_gbc::SAMPLE_RATE,
            cycles_per_frame: ironboyadvance_gbc::CYCLES_PER_FRAME,
        },
    }
}

pub fn boot(
    kind: Console,
    rom_path: &str,
    rom: Vec<u8>,
    bios: Vec<u8>,
    unix_seconds: u64,
    show_logs: bool,
) -> Result<Box<dyn System>, BootError> {
    match kind {
        Console::Gba => Ok(Box::new(GameBoyAdvance::new(
            PathBuf::from(rom_path),
            rom,
            bios,
            unix_seconds,
            show_logs,
            Interpreter::new(),
        )?)),
        Console::Gb | Console::Gbc => Ok(Box::new(GameBoyColor::new(
            kind,
            PathBuf::from(rom_path),
            rom,
            bios,
            show_logs,
        )?)),
    }
}
