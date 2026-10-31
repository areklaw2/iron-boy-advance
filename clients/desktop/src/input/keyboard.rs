use ironboyadvance::KeypadButton;
use winit::keyboard::{KeyCode, ModifiersState};

pub(crate) enum HotKey {
    Reset,
    TogglePause,
    ToggleMaxSpeed,
    ToggleFps,
    Screenshot,
}

pub(crate) fn keycode_to_hotkey(modifiers: ModifiersState, code: KeyCode) -> Option<HotKey> {
    let command = modifiers.super_key();
    match (command, code) {
        (true, KeyCode::KeyR) => Some(HotKey::Reset),
        (true, KeyCode::KeyP) => Some(HotKey::TogglePause),
        (false, KeyCode::F2) => Some(HotKey::ToggleMaxSpeed),
        (false, KeyCode::F3) => Some(HotKey::ToggleFps),
        (false, KeyCode::F4) => Some(HotKey::Screenshot),
        _ => None,
    }
}

pub(crate) fn keycode_to_button(code: KeyCode) -> Option<KeypadButton> {
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
