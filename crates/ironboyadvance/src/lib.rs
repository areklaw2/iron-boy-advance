use std::path::PathBuf;

use ironboyadvance_arm7tdmi_interpreter::Interpreter;
use ironboyadvance_gba::{GameBoyAdvance, GbaError};
use ironboyadvance_gbc::{GameBoyColor, GbcError};
use thiserror::Error;

pub use ironboyadvance_common::keypad::KeypadButton;
pub use ironboyadvance_common::system::{Console, System, detect_console};

#[derive(Error, Debug)]
pub enum BootError {
    #[error("failed to initialize GBA: {0}")]
    Gba(#[from] GbaError),
    #[error("failed to initialize GBC: {0}")]
    Gbc(#[from] GbcError),
    #[error("unrecognized rom format")]
    UnknownFormat,
}

pub fn system_info(kind: Console) -> (usize, usize, f32, u32, usize) {
    match kind {
        Console::Gba => (
            ironboyadvance_gba::VIEWPORT_WIDTH,
            ironboyadvance_gba::VIEWPORT_HEIGHT,
            ironboyadvance_gba::FPS,
            ironboyadvance_gba::APU_SAMPLING_FREQUENCY as u32,
            ironboyadvance_gba::CYCLES_PER_FRAME,
        ),
        Console::Gb | Console::Gbc => (
            ironboyadvance_gbc::VIEWPORT_WIDTH,
            ironboyadvance_gbc::VIEWPORT_HEIGHT,
            ironboyadvance_gbc::FPS,
            ironboyadvance_gbc::SAMPLE_RATE,
            ironboyadvance_gbc::CYCLES_PER_FRAME,
        ),
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
