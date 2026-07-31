use std::path::PathBuf;

use ironboyadvance_gba::{GameBoyAdvance, GbaError};
use ironboyadvance_gbc::{GameBoyColor, GbcError};
use thiserror::Error;

pub use ironboyadvance_common::emulator::{Emulator, System, detect_system};
pub use ironboyadvance_gba::KeypadButton;

#[derive(Error, Debug)]
pub enum BootError {
    #[error("failed to initialize GBA: {0}")]
    Gba(#[from] GbaError),
    #[error("failed to initialize GBC: {0}")]
    Gbc(#[from] GbcError),
    #[error("unrecognized rom format")]
    UnknownFormat,
}

pub fn system_info(kind: System) -> (usize, usize, f32, u32, usize) {
    match kind {
        System::Gba => (
            ironboyadvance_gba::VIEWPORT_WIDTH,
            ironboyadvance_gba::VIEWPORT_HEIGHT,
            ironboyadvance_gba::FPS,
            ironboyadvance_gba::APU_SAMPLING_FREQUENCY as u32,
            ironboyadvance_gba::CYCLES_PER_FRAME,
        ),
        System::Gb | System::Gbc => (
            ironboyadvance_gbc::VIEWPORT_WIDTH,
            ironboyadvance_gbc::VIEWPORT_HEIGHT,
            ironboyadvance_gbc::FPS,
            ironboyadvance_gbc::SAMPLE_RATE,
            ironboyadvance_gbc::CYCLES_PER_FRAME,
        ),
    }
}

pub fn boot(
    kind: System,
    rom_path: &str,
    rom: Vec<u8>,
    bios: Vec<u8>,
    show_logs: bool,
) -> Result<Box<dyn Emulator>, BootError> {
    match kind {
        System::Gba => Ok(Box::new(GameBoyAdvance::new(PathBuf::from(rom_path), rom, bios, show_logs)?)),
        System::Gb | System::Gbc => Ok(Box::new(GameBoyColor::new(PathBuf::from(rom_path), rom, show_logs)?)),
    }
}
