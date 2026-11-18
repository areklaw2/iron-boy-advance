use std::path::PathBuf;

use ironboyadvance_arm7tdmi_interpreter::Interpreter;
#[cfg(target_arch = "aarch64")]
use ironboyadvance_arm7tdmi_jit::JitCompiler;
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
    #[error("the JIT is not supported on this platform")]
    JitUnsupported,
    #[cfg(target_arch = "aarch64")]
    #[error("failed to initialize JIT: {0}")]
    Jit(#[from] ironboyadvance_arm7tdmi_jit::JitError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Interpreter,
    Jit,
}

pub const GBA_VIEWPORT: Viewport = Viewport {
    width: ironboyadvance_gba::VIEWPORT_WIDTH,
    height: ironboyadvance_gba::VIEWPORT_HEIGHT,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemInfo {
    pub viewport: Viewport,
    pub native_fps: f32,
    pub sample_rate: u32,
    pub cycles_per_frame: usize,
}

pub fn system_info(kind: Console) -> SystemInfo {
    match kind {
        Console::Gba => SystemInfo {
            viewport: GBA_VIEWPORT,
            native_fps: ironboyadvance_gba::FPS,
            sample_rate: ironboyadvance_gba::APU_SAMPLING_FREQUENCY as u32,
            cycles_per_frame: ironboyadvance_gba::CYCLES_PER_FRAME,
        },
        Console::Gb | Console::Gbc => SystemInfo {
            viewport: Viewport {
                width: ironboyadvance_gbc::VIEWPORT_WIDTH,
                height: ironboyadvance_gbc::VIEWPORT_HEIGHT,
            },
            native_fps: ironboyadvance_gbc::FPS,
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
    execution_mode: ExecutionMode,
) -> Result<Box<dyn System>, BootError> {
    match kind {
        Console::Gba => match execution_mode {
            ExecutionMode::Interpreter => Ok(Box::new(GameBoyAdvance::new(
                PathBuf::from(rom_path),
                rom,
                bios,
                unix_seconds,
                show_logs,
                Interpreter::new(),
            )?)),
            ExecutionMode::Jit => {
                #[cfg(target_arch = "aarch64")]
                {
                    Ok(Box::new(GameBoyAdvance::new(
                        PathBuf::from(rom_path),
                        rom,
                        bios,
                        unix_seconds,
                        show_logs,
                        JitCompiler::new()?,
                    )?))
                }
                #[cfg(not(target_arch = "aarch64"))]
                {
                    Err(BootError::JitUnsupported)
                }
            }
        },
        Console::Gb | Console::Gbc => Ok(Box::new(GameBoyColor::new(
            kind,
            PathBuf::from(rom_path),
            rom,
            bios,
            show_logs,
        )?)),
    }
}
