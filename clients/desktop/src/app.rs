use std::{collections::HashMap, path::Path, sync::Arc};

use ironboyadvance::GBA_VIEWPORT;
use muda::Menu;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    BASE_TITLE,
    audio::{self, AudioSink},
    config::Config,
    emulator::{self, Emulator},
    input::{Controller, HotKey, KeypadTracker, build_menu, keycode_to_button, keycode_to_hotkey},
    video::{GpuContext, Overlays, save_screenshot},
    window::WindowState,
};

const INITIAL_WINDOW_SCALE: u32 = 6;

#[derive(Debug)]
pub(crate) enum UserEvent {
    MenuEvent(muda::MenuEvent),
    EmulatorExited,
}

pub(crate) struct Application {
    config: Config,
    show_logs: bool,
    show_fps: bool,
    emulator: Option<Emulator>,
    rom_path: Option<String>,
    _audio_stream: Option<cpal::Stream>,
    audio_sink: Option<AudioSink>,
    gpu: Option<GpuContext>,
    windows: HashMap<WindowId, WindowState>,
    main_window: Option<WindowId>,
    focused_window: Option<WindowId>,
    controller: Option<Controller>,
    keypad_tracker: KeypadTracker,
    modifiers: ModifiersState,
    menu: Option<Menu>,
    proxy: EventLoopProxy<UserEvent>,
}

impl Application {
    pub(crate) fn new(rom_path: Option<String>, config: Config, show_logs: bool, proxy: EventLoopProxy<UserEvent>) -> Self {
        let (_audio_stream, audio_sink) = match audio::start() {
            Some(output) => (Some(output.stream), Some(output.sink)),
            None => (None, None),
        };

        Self {
            config,
            show_logs,
            show_fps: false,
            emulator: None,
            rom_path,
            _audio_stream,
            audio_sink,
            gpu: None,
            windows: HashMap::new(),
            main_window: None,
            focused_window: None,
            controller: None,
            keypad_tracker: KeypadTracker::new(),
            modifiers: ModifiersState::empty(),
            menu: None,
            proxy,
        }
    }
    fn mount(&mut self, rom_path: String) {
        if let Some(emulator) = self.emulator.take() {
            self.audio_sink = emulator.stop();
        }

        let rom_buffer = match emulator::read_rom(&rom_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("failed to read rom {rom_path}: {e}");
                return;
            }
        };

        match Emulator::spawn(
            rom_path.clone(),
            rom_buffer,
            &self.config,
            self.show_logs,
            self.proxy.clone(),
            self.audio_sink.take(),
        ) {
            Ok(emulator) => self.emulator = Some(emulator),
            Err(failure) => {
                tracing::error!("failed to load rom {rom_path}: {}", failure.error);
                self.audio_sink = failure.audio_sink;
            }
        }
    }

    fn attach_to_main_window(&mut self) {
        let Some(gpu) = self.gpu.as_ref() else { return };
        let Some(window_id) = self.main_window else { return };
        let Some(state) = self.windows.get_mut(&window_id) else {
            return;
        };

        match self.emulator.as_ref() {
            Some(emulator) => {
                let rom_name = Path::new(emulator.rom_path())
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(emulator.rom_path());
                state.window.set_title(&format!("{BASE_TITLE} - {rom_name}"));
                state.attach_renderer(gpu, emulator.viewport());
            }
            None => {
                state.window.set_title(BASE_TITLE);
                state.frame_renderer = None;
            }
        }
    }

    fn load_rom(&mut self, rom_path: String) {
        self.mount(rom_path);
        self.attach_to_main_window();
    }

    fn main_window_state(&self) -> Option<&WindowState> {
        self.main_window.and_then(|window_id| self.windows.get(&window_id))
    }

    fn handle_hotkey(&mut self, code: KeyCode, state: ElementState) -> bool {
        if state != ElementState::Pressed {
            return false;
        }

        let Some(hotkey) = keycode_to_hotkey(self.modifiers, code) else {
            return false;
        };

        match hotkey {
            HotKey::Screenshot => {
                if let (Some(emulator), Some(window_state)) = (self.emulator.as_ref(), self.main_window_state()) {
                    match emulator.last_frame() {
                        Some(frame) => {
                            let window_size = window_state.window.inner_size();
                            if let Err(e) = save_screenshot(frame, emulator.viewport(), window_size) {
                                tracing::error!("screenshot failed: {e}");
                            }
                        }
                        None => tracing::warn!("screenshot requested before any frame has arrived"),
                    }
                }
            }
            HotKey::ToggleFps => {
                if self.emulator.is_some() {
                    self.show_fps ^= true;
                }
            }
            HotKey::TogglePause => {
                if let Some(emulator) = self.emulator.as_mut() {
                    emulator.toggle_pause();
                }
            }
            HotKey::ToggleMaxSpeed => {
                if let Some(emulator) = self.emulator.as_mut() {
                    emulator.toggle_max_speed();
                }
            }
            HotKey::Reset => {
                if let Some(rom_path) = self.emulator.as_ref().map(|emulator| emulator.rom_path().to_owned()) {
                    self.load_rom(rom_path);
                }
            }
        }

        true
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.windows.is_empty() {
            return;
        }

        if let Some(rom_path) = self.rom_path.take() {
            self.mount(rom_path);
        }

        let viewport = self.emulator.as_ref().map(Emulator::viewport).unwrap_or(GBA_VIEWPORT);
        let attrs = Window::default_attributes()
            .with_title(BASE_TITLE)
            .with_inner_size(LogicalSize::new(
                viewport.width as u32 * INITIAL_WINDOW_SCALE,
                viewport.height as u32 * INITIAL_WINDOW_SCALE,
            ));
        let window = Arc::new(event_loop.create_window(attrs).expect("failed to create window"));

        if self.gpu.is_none() {
            self.gpu = Some(pollster::block_on(GpuContext::new()));
        }
        let gpu = self.gpu.as_ref().unwrap();

        let window_id = window.id();
        self.windows.insert(window_id, WindowState::new(gpu, window));
        self.main_window = Some(window_id);
        self.attach_to_main_window();

        self.controller = Controller::new();
        self.menu = Some(build_menu(self.proxy.clone()));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(state) = self.windows.get_mut(&window_id) else {
            return;
        };
        let window = state.window.clone();
        let keypad = self
            .emulator
            .as_ref()
            .filter(|_| self.main_window == Some(window_id))
            .map(|emulator| emulator.keypad().clone());

        let egui_consumed = state.egui_layer.on_window_event(&window, &event);

        match event {
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
                if self.main_window == Some(window_id) {
                    event_loop.exit();
                }
            }

            WindowEvent::Focused(true) => self.focused_window = Some(window_id),
            WindowEvent::Focused(false) => {
                if self.focused_window == Some(window_id) {
                    self.focused_window = None;
                    if let Some(emulator) = self.emulator.as_ref() {
                        self.keypad_tracker.release_controller(emulator.keypad());
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::Resized(size) => {
                if let (Some(gpu), Some(state)) = (self.gpu.as_ref(), self.windows.get_mut(&window_id)) {
                    state.resize(gpu, size);
                }
                window.request_redraw();
            }
            WindowEvent::DroppedFile(path) => {
                if let Some(rom_path) = path.to_str() {
                    self.load_rom(rom_path.to_string());
                } else {
                    tracing::error!("dropped file path was not valid UTF-8: {}", path.display());
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                if code == KeyCode::Escape && key_state == ElementState::Pressed {
                    event_loop.exit();
                    return;
                }

                if self.handle_hotkey(code, key_state) {
                    return;
                }

                if !egui_consumed
                    && let Some(button) = keycode_to_button(code)
                    && let Some(keypad) = keypad.as_ref()
                {
                    self.keypad_tracker
                        .handle_keyboard_button(button, key_state == ElementState::Pressed, keypad);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(emulator) = self.emulator.as_mut() {
                    emulator.drain_frames();
                }

                let overlays = Overlays {
                    fps: self.emulator.as_ref().filter(|_| self.show_fps).map(Emulator::fps),
                    paused: self.emulator.as_ref().is_some_and(Emulator::paused),
                };

                if let (Some(gpu), Some(state)) = (self.gpu.as_ref(), self.windows.get_mut(&window_id)) {
                    state.render(gpu, self.emulator.as_ref(), &overlays);
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::MenuEvent(menu_event) => tracing::debug!("menu event {:?}", menu_event.id()),
            UserEvent::EmulatorExited => {
                if self.emulator.as_ref().is_some_and(Emulator::panicked)
                    && let Some(emulator) = self.emulator.take()
                {
                    self.audio_sink = emulator.stop();
                    if self.audio_sink.is_none()
                        && let Some(output) = audio::start()
                    {
                        self._audio_stream = Some(output.stream);
                        self.audio_sink = Some(output.sink);
                    }
                    self.attach_to_main_window();
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(controller) = self.controller.as_mut() {
            let buttons = controller.poll();
            if self.focused_window.is_some()
                && let Some(emulator) = self.emulator.as_ref()
            {
                for (button, pressed) in buttons {
                    self.keypad_tracker
                        .handle_controller_button(button, pressed, emulator.keypad());
                }
            }
        }

        for state in self.windows.values() {
            state.window.request_redraw();
        }
    }
}
