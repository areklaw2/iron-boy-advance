use ironboyadvance_gba::KeypadButton;
use objc2_game_controller::{GCController, GCExtendedGamepad};

use super::ControllerBackend;

pub struct MacosControllerBackend;

impl MacosControllerBackend {
    pub fn new() -> Option<Self> {
        unsafe { GCController::setShouldMonitorBackgroundEvents(true) };
        Some(Self)
    }
}

impl ControllerBackend for MacosControllerBackend {
    fn poll(&mut self) -> Vec<(KeypadButton, bool)> {
        let pad = unsafe { GCController::controllers() }
            .firstObject()
            .and_then(|controller| unsafe { controller.extendedGamepad() });

        match pad.as_deref() {
            Some(pad) => button_states(pad).to_vec(),
            None => BUTTONS.map(|button| (button, false)).to_vec(),
        }
    }
}

const BUTTONS: [KeypadButton; 10] = [
    KeypadButton::A,
    KeypadButton::B,
    KeypadButton::L,
    KeypadButton::R,
    KeypadButton::Start,
    KeypadButton::Select,
    KeypadButton::Up,
    KeypadButton::Down,
    KeypadButton::Left,
    KeypadButton::Right,
];

/// Reads the current state of every button on `pad`. Position-based face
/// buttons: East (buttonB) -> A, South (buttonA) -> B, matching the gilrs backend.
fn button_states(pad: &GCExtendedGamepad) -> [(KeypadButton, bool); 10] {
    unsafe {
        let dpad = pad.dpad();
        let select = match pad.buttonOptions() {
            Some(button) => button.isPressed(),
            None => false,
        };
        [
            (KeypadButton::A, pad.buttonB().isPressed()),
            (KeypadButton::B, pad.buttonA().isPressed()),
            (KeypadButton::L, pad.leftShoulder().isPressed()),
            (KeypadButton::R, pad.rightShoulder().isPressed()),
            (KeypadButton::Start, pad.buttonMenu().isPressed()),
            (KeypadButton::Select, select),
            (KeypadButton::Up, dpad.up().isPressed()),
            (KeypadButton::Down, dpad.down().isPressed()),
            (KeypadButton::Left, dpad.left().isPressed()),
            (KeypadButton::Right, dpad.right().isPressed()),
        ]
    }
}
