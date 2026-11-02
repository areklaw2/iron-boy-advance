use std::io;
use thiserror::Error;
use winit::event_loop::EventLoop;

mod app;
mod audio;
mod config;
mod emulator;
mod input;
mod logger;
mod menu;
mod screenshot;
mod video;
mod window;

use crate::{
    app::{Application, UserEvent},
    config::Config,
    logger::initialize_logger,
};
use ironboyadvance::{Console, detect_console};

const BASE_TITLE: &str = "Iron Boy Advance";

#[derive(Error, Debug)]
pub enum DesktopError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Failed to create event loop: {0}")]
    EventLoopError(#[from] winit::error::EventLoopError),
    #[error("Screenshot failed: {0}")]
    ScreenshotError(#[from] image::ImageError),
    #[error("Failed to boot emulator: {0}")]
    BootError(#[from] ironboyadvance::BootError),
    #[error("Emulator thread exited before it finished booting")]
    EmulatorThreadExited,
}

pub fn run(rom_path: Option<String>, bios_path: Option<String>, show_logs: bool) -> Result<(), DesktopError> {
    let _log_guard = if show_logs { Some(initialize_logger()) } else { None };

    let mut config = Config::load().unwrap_or_default();

    if let Some(bios_path) = &bios_path {
        let console = match rom_path.as_deref() {
            Some(path) => emulator::read_rom(path).ok().and_then(|rom| detect_console(&rom)),
            None => Some(Console::Gba),
        };
        match console {
            Some(console) => {
                config.set_bios(console, bios_path);
                if let Err(e) = config.save() {
                    tracing::warn!("failed to persist boot rom path to config: {e}");
                }
            }
            None => tracing::warn!("not saving --bios: could not detect a console from --rom"),
        }
    }

    let mut app = Application::new(rom_path, config, show_logs);

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    muda::MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));
    event_loop.run_app(&mut app)?;

    std::process::exit(0);
}
