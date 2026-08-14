use std::{io, path::Path};
use thiserror::Error;
use winit::event_loop::EventLoop;

mod app;
mod audio;
mod controller;
mod emulator;
mod frame;
mod gpu;
mod input;
mod logger;
mod windows;

use crate::{app::Application, logger::initialize_logger};

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

    let (title, initial_emulator) = match rom_path {
        Some(rom_path) => {
            let rom_name = Path::new(&rom_path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .ok_or(DesktopError::InvalidRomPath)?;
            let emu = emulator::spawn(rom_path, bios_path, show_logs)?;
            (format!("{BASE_TITLE} - {rom_name}"), Some(emu))
        }
        None => (BASE_TITLE.to_string(), None),
    };

    let mut app = Application::new(title, initial_emulator, show_logs);

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;

    std::process::exit(0);
}
