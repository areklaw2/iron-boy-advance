use std::{fs, io, path::Path};
use thiserror::Error;
use winit::event_loop::EventLoop;

mod app;
mod emulator;
mod frame;
mod gui;
mod input;
mod logger;
mod renderer;

use crate::{app::Application, logger::initialize_logger};

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
}

pub fn run(rom_path: String, bios_path: Option<String>, show_logs: bool) -> Result<(), DesktopError> {
    let _log_guard = if show_logs { Some(initialize_logger()) } else { None };

    let rom_name = Path::new(&rom_path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
        .ok_or(DesktopError::InvalidRomPath)?;

    let rom_buffer = fs::read(&rom_path)?;
    let bios_buffer: Box<[u8]> = match bios_path {
        Some(path) => fs::read(path)?.into_boxed_slice(),
        None => Box::default(),
    };

    let emu = emulator::spawn(rom_buffer, bios_buffer, show_logs);

    let title = format!("Iron Boy Advance - {rom_name}");
    let mut app = Application::new(title, emu);

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}
