use std::collections::HashSet;

use ironboyadvance_core::{GameBoyAdvance, GbaError, KeypadButton};

use sdl2::{
    EventPump,
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
}

pub struct Application {
    game_boy_advance: GameBoyAdvance,
    window_manager: WindowManager,
    event_pump: EventPump,
    frame_timer: FrameTimer,
    overshoot: usize,
    pressed_keys: HashSet<KeypadButton>,
}

impl Application {
    pub fn new(rom_path: String, bios_path: Option<String>, show_logs: bool) -> Result<Application, ApplicationError> {
        if show_logs {
            initilize_logger();
        }

        let bios_path = match bios_path {
            Some(bios_path) => Some(bios_path.into()),
            None => None,
        };

        let game_boy_advance = GameBoyAdvance::new(rom_path.into(), bios_path, show_logs)?;

        let sdl_context = sdl2::init().map_err(ApplicationError::SdlInitError)?;
        let window_manager = WindowManager::new(&sdl_context, game_boy_advance.rom_name())?;
        let event_pump = sdl_context.event_pump().map_err(ApplicationError::EventPumpError)?;

        Ok(Self {
            game_boy_advance,
            window_manager,
            event_pump,
            frame_timer: FrameTimer::new(),
            overshoot: 0,
            pressed_keys: HashSet::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), ApplicationError> {
        let main_window_id = self.window_manager.main_canvas().window().id();

        'game: loop {
            for event in self.event_pump.poll_iter() {
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
                        if let Some(button) = keycode_to_button(keycode) {
                            self.pressed_keys.insert(button);
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(keycode), ..
                    } => {
                        if let Some(button) = keycode_to_button(keycode) {
                            self.pressed_keys.remove(&button);
                        }
                    }
                    _ => {}
                };
            }

            self.run_game_boy()?;
        }

        Ok(())
    }

    fn run_game_boy(&mut self) -> Result<(), ApplicationError> {
        self.overshoot = self.game_boy_advance.run(self.overshoot, &self.pressed_keys);
        let fps = self.frame_timer.fps();
        self.window_manager
            .render_screen(self.game_boy_advance.frame_buffer(), Some(fps))?;
        self.frame_timer.slow_frame();
        self.frame_timer.count_frame();
        Ok(())
    }
}

fn keycode_to_button(keycode: Keycode) -> Option<KeypadButton> {
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
