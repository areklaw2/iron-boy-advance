use std::{
    fs, io,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU16, Ordering},
        mpsc,
    },
    thread,
};

use ironboyadvance_core::{CYCLES_PER_FRAME, GameBoyAdvance, GbaError, KeypadButton};

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
};
use thiserror::Error;

use crate::{
    frame::FrameTimer,
    logger::initilize_logger,
    window::{WindowError, WindowManager},
};

mod frame;
mod logger;
mod window;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Failed to initialize SDL context: {0}")]
    SdlInitError(String),
    #[error("There was a window error: {0}")]
    WindowError(#[from] WindowError),
    #[error("Failed to initialize event pump: {0}")]
    EventPumpError(String),
    #[error("There was a game boy error: {0}")]
    GbaError(#[from] GbaError),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Rom path was invalid")]
    InvalidRomPath,
}

pub fn run(rom_path: String, bios_path: Option<String>, show_logs: bool) -> Result<(), ApplicationError> {
    if show_logs {
        initilize_logger();
    }

    let rom_name = Path::new(&rom_path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
        .ok_or(ApplicationError::InvalidRomPath)?;

    let sdl_context = sdl2::init().map_err(ApplicationError::SdlInitError)?;
    let mut window_manager = WindowManager::new(&sdl_context, &rom_name)?;
    let mut event_pump = sdl_context.event_pump().map_err(ApplicationError::EventPumpError)?;

    let rom_buffer = fs::read(&rom_path)?;
    let bios_buffer: Box<[u8]> = match bios_path {
        Some(path) => fs::read(path)?.into_boxed_slice(),
        None => Box::default(),
    };

    let keypad_state = Arc::new(AtomicU16::new(0x03FF));
    let (frame_tx, frame_rx) = mpsc::channel::<Vec<u32>>();

    let emu_keypad = keypad_state.clone();
    thread::spawn(move || {
        let mut gba = match GameBoyAdvance::new(rom_buffer, bios_buffer, show_logs) {
            Ok(gba) => gba,
            Err(e) => panic!("Failed to initialize GBA: {e}"),
        };
        let mut overshoot = 0;
        let mut frame_timer = FrameTimer::new();

        const INPUT_SUBFRAMES: usize = 4;
        const CYCLES_PER_SUBFRAME: usize = CYCLES_PER_FRAME / INPUT_SUBFRAMES;

        loop {
            for _ in 0..INPUT_SUBFRAMES {
                gba.handle_pressed_buttons(emu_keypad.load(Ordering::Relaxed));
                overshoot = gba.run(CYCLES_PER_SUBFRAME, overshoot);
            }
            let _ = frame_tx.send(gba.frame_buffer().to_vec());
            frame_timer.slow_frame();
            frame_timer.count_frame();
        }
    });

    let main_window_id = window_manager.main_canvas().window().id();
    let mut keypad_bits: u16 = 0x03FF;
    let mut last_frame: Option<Vec<u32>> = None;
    let mut frame_timer = FrameTimer::new();

    'game: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'game,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    window_id,
                    ..
                } => {
                    if window_id == main_window_id {
                        break 'game;
                    }
                }
                Event::Window {
                    win_event: WindowEvent::Close,
                    window_id,
                    ..
                } => {
                    if window_id == main_window_id {
                        break 'game;
                    }
                }
                Event::KeyDown {
                    keycode: Some(keycode), ..
                } => {
                    if let Some(bit) = keycode_to_bit(keycode) {
                        keypad_bits &= !(1 << bit as u16);
                        keypad_state.store(keypad_bits, Ordering::Relaxed);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode), ..
                } => {
                    if let Some(bit) = keycode_to_bit(keycode) {
                        keypad_bits |= 1 << bit as u16;
                        keypad_state.store(keypad_bits, Ordering::Relaxed);
                    }
                }
                _ => {}
            };
        }

        let mut new_frame = false;
        while let Ok(frame) = frame_rx.try_recv() {
            last_frame = Some(frame);
            frame_timer.count_frame();
            new_frame = true;
        }
        if new_frame {
            if let Some(ref fb) = last_frame {
                window_manager.render_screen(fb, Some(frame_timer.fps()))?;
            }
        }
    }

    Ok(())
}

fn keycode_to_bit(keycode: Keycode) -> Option<KeypadButton> {
    match keycode {
        Keycode::X => Some(KeypadButton::A),
        Keycode::Z => Some(KeypadButton::B),
        Keycode::Backspace => Some(KeypadButton::Select),
        Keycode::Return => Some(KeypadButton::Start),
        Keycode::Up => Some(KeypadButton::Up),
        Keycode::Left => Some(KeypadButton::Left),
        Keycode::Down => Some(KeypadButton::Down),
        Keycode::Right => Some(KeypadButton::Right),
        Keycode::S => Some(KeypadButton::R),
        Keycode::A => Some(KeypadButton::L),
        _ => None,
    }
}
