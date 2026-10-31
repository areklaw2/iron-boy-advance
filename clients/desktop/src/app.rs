use std::{collections::HashMap, sync::Arc};

use egui_wgpu::ScreenDescriptor;
use muda::Menu;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    BASE_TITLE,
    config::Config,
    emulator::{self, EmulatorCommand, EmulatorHandle, FpsCounter},
    input::{Controller, HotKey, KeypadTracker, build_menu, keycode_to_button, keycode_to_hotkey},
    screenshot::save_screenshot,
    video::{GpuContext, draw_splash},
    window::{ScreenContent, WindowState},
};

#[derive(Debug)]
pub(crate) enum UserEvent {
    MenuEvent(muda::MenuEvent),
}

pub(crate) struct Application {
    show_logs: bool,
    config: Config,
    keypad_tracker: KeypadTracker,
    modifiers: ModifiersState,
    gpu: Option<GpuContext>,
    windows: HashMap<WindowId, WindowState>,
    main_window: Option<WindowId>,
    controller: Option<Controller>,
    menu: Option<Menu>,
    initial_emulator: Option<EmulatorHandle>,
}

impl Application {
    pub(crate) fn new(_title: String, initial_emulator: Option<EmulatorHandle>, config: Config, show_logs: bool) -> Self {
        Self {
            show_logs,
            config,
            keypad_tracker: KeypadTracker::new(),
            modifiers: ModifiersState::empty(),
            gpu: None,
            windows: HashMap::new(),
            main_window: None,
            controller: None,
            menu: None,
            initial_emulator,
        }
    }

    fn load_rom(&mut self, window_id: WindowId, rom_path: String) {
        let rom_buffer = match emulator::read_rom(&rom_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("failed to read rom {rom_path}: {e}");
                return;
            }
        };
        match emulator::spawn(rom_path.clone(), rom_buffer, self.config.clone(), self.show_logs) {
            Ok(handle) => {
                let Some(gpu) = self.gpu.as_ref() else { return };
                let Some(state) = self.windows.get_mut(&window_id) else {
                    return;
                };

                let rom_name = std::path::Path::new(&rom_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&rom_path);
                state.start_running(gpu, handle, rom_name);
            }
            Err(e) => tracing::error!("failed to load rom {rom_path}: {e}"),
        }
    }

    fn drain_and_render(&mut self, window_id: WindowId) {
        let (Some(gpu), Some(state)) = (self.gpu.as_ref(), self.windows.get_mut(&window_id)) else {
            return;
        };

        state.drain_frames();

        let output = match state.surface.acquire() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let size = state.window.inner_size();
                state.resize(gpu, size);
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Validation => {
                tracing::error!("wgpu surface validation error during acquire");
                return;
            }
        };

        if let Some(running) = state.content.running()
            && let Some(fb) = &running.last_frame
        {
            running.renderer.upload_frame(gpu, fb);
        }

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("desktop-encoder"),
        });

        let screen = ScreenDescriptor {
            size_in_pixels: [state.surface.config().width, state.surface.config().height],
            pixels_per_point: state.window.scale_factor() as f32,
        };

        let is_splash = matches!(state.content, ScreenContent::Splash);

        // Egui must upload its textures and vertex/index buffers BEFORE the render pass
        // begins, because begin_render_pass borrows the encoder.
        let prepared = state
            .gui
            .prepare(&state.window, gpu.device(), gpu.queue(), &mut encoder, &screen, |ui| {
                if is_splash {
                    draw_splash(ui);
                }
            });

        {
            let mut rpass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("desktop-rpass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                })
                .forget_lifetime();

            if let Some(running) = state.content.running()
                && running.last_frame.is_some()
            {
                running.renderer.draw_frame(&mut rpass);
            }

            state.gui.paint(&mut rpass, &prepared, &screen);
        }

        state.gui.cleanup(prepared);

        gpu.queue().submit(std::iter::once(encoder.finish()));
        gpu.queue().present(output);
    }

    fn main_window_state(&self) -> Option<&WindowState> {
        self.main_window.and_then(|window_id| self.windows.get(&window_id))
    }

    fn main_window_state_mut(&mut self) -> Option<&mut WindowState> {
        self.main_window.and_then(|window_id| self.windows.get_mut(&window_id))
    }

    fn send_emulator_command(&self, command: EmulatorCommand) {
        if let Some(window_state) = self.main_window_state()
            && let Some(running) = window_state.content.running()
        {
            running.emulator.send(command);
        }
    }

    fn handle_hotkey(&mut self, code: KeyCode, state: ElementState) -> bool {
        if state != ElementState::Pressed {
            return false;
        }

        let Some(hotkey) = keycode_to_hotkey(self.modifiers, code) else {
            return false;
        };

        match hotkey {
            HotKey::ToggleFps => {
                if let Some(window_state) = self.main_window_state_mut()
                    && window_state.content.running().is_some()
                {
                    *window_state.gui.fps_overlay_mut().show_mut() ^= true;
                }
            }

            HotKey::Screenshot => {
                if let Some(window_state) = self.main_window_state()
                    && let Some(running) = window_state.content.running()
                {
                    let surface_config = window_state.surface.config();
                    if let Err(e) = save_screenshot(running, surface_config.width, surface_config.height) {
                        tracing::error!("screenshot failed: {e}");
                    }
                }
            }
            HotKey::TogglePause => {
                if let Some(running) = self.main_window_state_mut().and_then(|state| state.content.running_mut()) {
                    running.paused = !running.paused;
                    running.emulator.send(EmulatorCommand::SetPaused(running.paused));
                }
            }
            //TODO: add opt-in pause-on-minimize/unfocus config
            HotKey::ToggleMaxSpeed => {
                if let Some(running) = self.main_window_state_mut().and_then(|state| state.content.running_mut()) {
                    running.max_speed = !running.max_speed;
                    running.emulator.send(EmulatorCommand::SetMaxSpeed(running.max_speed));
                }
            }
            HotKey::Reset => {
                self.send_emulator_command(EmulatorCommand::Reset);
                if let Some(window_state) = self.main_window_state_mut()
                    && let Some(running) = window_state.content.running_mut()
                {
                    running.last_frame = None;
                    running.fps_counter = FpsCounter::new();
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

        let initial_viewport = self
            .initial_emulator
            .as_ref()
            .map(|e| (e.viewport_width, e.viewport_height))
            .unwrap_or((240, 160));

        let attrs = Window::default_attributes()
            .with_title(BASE_TITLE)
            .with_inner_size(LogicalSize::new(initial_viewport.0 as u32 * 6, initial_viewport.1 as u32 * 6));
        let window = Arc::new(event_loop.create_window(attrs).expect("failed to create window"));

        if self.gpu.is_none() {
            self.gpu = Some(pollster::block_on(GpuContext::new()));
        }
        let gpu = self.gpu.as_ref().unwrap();

        let window_id = window.id();
        self.windows
            .insert(window_id, WindowState::new(gpu, window, self.initial_emulator.take()));
        self.main_window = Some(window_id);
        self.controller = Controller::new();
        self.menu = Some(build_menu());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(state) = self.windows.get_mut(&window_id) else {
            return;
        };
        let window = state.window.clone();
        let keypad = state.content.running().map(|r| r.emulator.keypad.clone());

        let egui_consumed = state.gui.on_window_event(&window, &event);

        match event {
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
                if self.main_window == Some(window_id) {
                    event_loop.exit();
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
                    self.load_rom(window_id, rom_path.to_string());
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
                self.drain_and_render(window_id);
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        let UserEvent::MenuEvent(menu_event) = event;
        tracing::debug!("menu event {:?}", menu_event.id());
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let keypad = self
            .main_window_state()
            .and_then(|state| state.content.running())
            .map(|running| running.emulator.keypad.clone());

        if let (Some(controller), Some(keypad)) = (self.controller.as_mut(), keypad) {
            for (button, pressed) in controller.poll() {
                self.keypad_tracker.handle_controller_button(button, pressed, &keypad);
            }
        }

        for state in self.windows.values() {
            state.window.request_redraw();
        }
    }
}
