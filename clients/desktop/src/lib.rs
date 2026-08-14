use std::{io, path::Path};
use thiserror::Error;
use winit::event_loop::EventLoop;

mod app;
mod audio;
mod config;
mod controller;
mod emulator;
mod frame;
mod gpu;
mod input;
mod logger;
mod windows;

use crate::{app::Application, config::Config, logger::initialize_logger};
use ironboyadvance::{System, detect_system};

const BASE_TITLE: &str = "Iron Boy Advance";

#[derive(Error, Debug)]
pub enum DesktopError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Rom path was invalid")]
    InvalidRomPath,
    #[error("Failed to create event loop: {0}")]
    EventLoopError(#[from] winit::error::EventLoopError),
    #[error("Screenshot failed: {0}")]
    ScreenshotError(#[from] image::ImageError),
    #[error("Failed to boot emulator: {0}")]
    BootError(#[from] ironboyadvance::BootError),
}

pub fn run(rom_path: Option<String>, bios_path: Option<String>, show_logs: bool) -> Result<(), DesktopError> {
    let _log_guard = if show_logs { Some(initialize_logger()) } else { None };

    let mut config = Config::load().unwrap_or_default();

    let rom_buffer = rom_path.as_deref().map(emulator::read_rom).transpose()?;
    let kind = rom_buffer.as_deref().and_then(detect_system);

    if let Some(ref boot_rom_arg) = bios_path {
        config.set_bios(kind.unwrap_or(System::Gba), boot_rom_arg);
        if let Err(e) = config.save() {
            tracing::warn!("failed to persist boot rom path to config: {e}");
        }
    }

    let (title, initial_emulator) = match rom_path.zip(rom_buffer) {
        Some((rom_path, rom_buffer)) => {
            let rom_name = Path::new(&rom_path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .ok_or(DesktopError::InvalidRomPath)?;
            let emu = emulator::spawn(rom_path, rom_buffer, config.clone(), show_logs)?;
            (format!("{BASE_TITLE} - {rom_name}"), Some(emu))
        }
        None => (BASE_TITLE.to_string(), None),
    };

    let mut app = Application::new(title, initial_emulator, config, show_logs);

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;

    std::process::exit(0);
}
