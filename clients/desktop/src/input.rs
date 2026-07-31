use std::sync::{
    Arc,
    atomic::{AtomicU16, Ordering},
};

use ironboyadvance::KeypadButton;
use winit::keyboard::{KeyCode, ModifiersState};

pub const KEYPAD_IDLE: u16 = 0x03FF;

pub enum HotKey {
    Reset,
    TogglePause,
    ToggleMaxSpeed,
    ToggleFpsOverlay,
    Screenshot,
}

pub fn keycode_to_hotkey(modifiers: ModifiersState, code: KeyCode) -> Option<HotKey> {
    let command = modifiers.super_key();
    match (command, code) {
        (true, KeyCode::KeyR) => Some(HotKey::Reset),
        (true, KeyCode::KeyP) => Some(HotKey::TogglePause),
        (false, KeyCode::F2) => Some(HotKey::ToggleMaxSpeed),
        (false, KeyCode::F3) => Some(HotKey::ToggleFpsOverlay),
        (false, KeyCode::F4) => Some(HotKey::Screenshot),
        _ => None,
    }
}

pub fn keycode_to_button(code: KeyCode) -> Option<KeypadButton> {
    match code {
        KeyCode::KeyX => Some(KeypadButton::A),
        KeyCode::KeyZ => Some(KeypadButton::B),
        KeyCode::Backspace => Some(KeypadButton::Select),
        KeyCode::Enter => Some(KeypadButton::Start),
        KeyCode::ArrowUp => Some(KeypadButton::Up),
        KeyCode::ArrowDown => Some(KeypadButton::Down),
        KeyCode::ArrowLeft => Some(KeypadButton::Left),
        KeyCode::ArrowRight => Some(KeypadButton::Right),
        KeyCode::KeyS => Some(KeypadButton::R),
        KeyCode::KeyA => Some(KeypadButton::L),
        _ => None,
    }
}

pub struct KeypadTracker {
    keyboard: u16,
    controller: u16,
}

impl KeypadTracker {
    pub fn new() -> Self {
        Self {
            keyboard: KEYPAD_IDLE,
            controller: KEYPAD_IDLE,
        }
    }

    pub fn handle_keyboard_button(&mut self, button: KeypadButton, pressed: bool, out: &Arc<AtomicU16>) {
        self.keyboard = apply(self.keyboard, button, pressed);
        self.store(out);
    }

    pub fn handle_controller_button(&mut self, button: KeypadButton, pressed: bool, out: &Arc<AtomicU16>) {
        self.controller = apply(self.controller, button, pressed);
        self.store(out);
    }

    fn store(&self, out: &Arc<AtomicU16>) {
        out.store(self.keyboard & self.controller, Ordering::Relaxed);
    }
}

fn apply(bits: u16, button: KeypadButton, pressed: bool) -> u16 {
    let mask = 1u16 << button as u16;
    match pressed {
        true => bits & !mask,
        false => bits | mask,
    }
}
