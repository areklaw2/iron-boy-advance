use std::sync::{
    Arc,
    atomic::{AtomicU16, Ordering},
};

use ironboyadvance_core::KeypadButton;
use winit::{event::ElementState, keyboard::KeyCode};

pub const KEYPAD_IDLE: u16 = 0x03FF;

pub enum HotKey {
    TogglePause,
    ToggleMaxSpeed,
    ToggleFpsOverlay,
    Screenshot,
}

pub fn keycode_to_hotkey(code: KeyCode) -> Option<HotKey> {
    match code {
        KeyCode::KeyP => Some(HotKey::TogglePause),
        KeyCode::KeyS => Some(HotKey::ToggleMaxSpeed),
        KeyCode::F3 => Some(HotKey::ToggleFpsOverlay),
        KeyCode::F4 => Some(HotKey::Screenshot),
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
    bits: u16,
}

impl KeypadTracker {
    pub fn new() -> Self {
        Self { bits: KEYPAD_IDLE }
    }

    pub fn handle_button(&mut self, code: KeyCode, state: ElementState, out: &Arc<AtomicU16>) {
        let Some(button) = keycode_to_button(code) else {
            return;
        };
        let mask = 1u16 << button as u16;
        match state {
            ElementState::Pressed => self.bits &= !mask,
            ElementState::Released => self.bits |= mask,
        }
        out.store(self.bits, Ordering::Relaxed);
    }
}
