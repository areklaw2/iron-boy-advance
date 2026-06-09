use ironboyadvance_core::KeypadButton;

#[cfg(not(target_os = "macos"))]
mod gilrs_backend;
#[cfg(target_os = "macos")]
mod macos_backend;

pub trait ControllerBackend {
    fn poll(&mut self) -> Vec<(KeypadButton, bool)>;
}

pub struct Controller {
    backend: Box<dyn ControllerBackend>,
}

impl Controller {
    pub fn new() -> Option<Controller> {
        #[cfg(target_os = "macos")]
        let backend = Box::new(macos_backend::MacosControllerBackend::new()?) as Box<dyn ControllerBackend>;
        #[cfg(not(target_os = "macos"))]
        let backend = Box::new(gilrs_backend::GilrsBackend::new()?) as Box<dyn ControllerBackend>;

        Some(Controller { backend })
    }

    pub fn poll(&mut self) -> Vec<(KeypadButton, bool)> {
        self.backend.poll()
    }
}
