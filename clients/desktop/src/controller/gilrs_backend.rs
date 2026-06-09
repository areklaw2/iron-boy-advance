use gilrs::{Button, EventType, Gilrs};
use ironboyadvance_core::KeypadButton;

use super::ControllerBackend;

pub struct GilrsBackend {
    gilrs: Gilrs,
}

impl GilrsBackend {
    pub fn new() -> Option<Self> {
        let gilrs = Gilrs::new().ok()?;
        Some(Self { gilrs })
    }
}

impl ControllerBackend for GilrsBackend {
    fn poll(&mut self) -> Vec<(KeypadButton, bool)> {
        let mut events = Vec::new();
        while let Some(event) = self.gilrs.next_event() {
            match event.event {
                EventType::ButtonPressed(button, _) => {
                    if let Some(button) = map(button) {
                        events.push((button, true));
                    }
                }
                EventType::ButtonReleased(button, _) => {
                    if let Some(button) = map(button) {
                        events.push((button, false));
                    }
                }
                _ => {}
            }
        }
        events
    }
}

fn map(button: Button) -> Option<KeypadButton> {
    match button {
        Button::East => Some(KeypadButton::A),
        Button::South => Some(KeypadButton::B),
        Button::LeftTrigger => Some(KeypadButton::L),
        Button::RightTrigger => Some(KeypadButton::R),
        Button::Start => Some(KeypadButton::Start),
        Button::Select => Some(KeypadButton::Select),
        Button::DPadUp => Some(KeypadButton::Up),
        Button::DPadDown => Some(KeypadButton::Down),
        Button::DPadLeft => Some(KeypadButton::Left),
        Button::DPadRight => Some(KeypadButton::Right),
        _ => None,
    }
}
