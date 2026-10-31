use ironboyadvance::KeypadButton;
use std::sync::{
    Arc,
    atomic::{AtomicU16, Ordering},
};

pub(crate) const KEYPAD_IDLE: u16 = 0x03FF;

pub(crate) struct KeypadTracker {
    keyboard: u16,
    controller: u16,
}

impl KeypadTracker {
    pub(crate) fn new() -> Self {
        Self {
            keyboard: KEYPAD_IDLE,
            controller: KEYPAD_IDLE,
        }
    }

    pub(crate) fn handle_keyboard_button(&mut self, button: KeypadButton, pressed: bool, out: &Arc<AtomicU16>) {
        self.keyboard = apply(self.keyboard, button, pressed);
        self.store(out);
    }

    pub(crate) fn handle_controller_button(&mut self, button: KeypadButton, pressed: bool, out: &Arc<AtomicU16>) {
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
