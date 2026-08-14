use std::{collections::HashMap, sync::Arc};

use egui_wgpu::ScreenDescriptor;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    BASE_TITLE, DesktopError,
    controller::Controller,
    emulator::{self, EmulatorCommand, EmulatorHandle},
    frame::FrameTimer,
    gpu::GpuContext,
    input::{HotKey, KeypadTracker, keycode_to_button, keycode_to_hotkey},
    windows::{GbaRenderer, Gui, WindowSurface, draw_splash},
};

struct RunningContent {
    emulator: EmulatorHandle,
    renderer: GbaRenderer,
    last_frame: Option<Vec<u32>>,
    fps_timer: FrameTimer,
}

enum ScreenContent {
    Splash,
    Running(Box<RunningContent>),
}

impl ScreenContent {
    fn running(&self) -> Option<&RunningContent> {
        match self {
            ScreenContent::Running(running) => Some(running),
            ScreenContent::Splash => None,
        }
    }

    fn running_mut(&mut self) -> Option<&mut RunningContent> {
        match self {
            ScreenContent::Running(running) => Some(running),
            ScreenContent::Splash => None,
        }
    }
}

struct WindowState {
    window: Arc<Window>,
    surface: WindowSurface,
    gui: Gui,
    content: ScreenContent,
}

pub struct Application {
    show_logs: bool,
    keypad_tracker: KeypadTracker,
    modifiers: ModifiersState,

    gpu: Option<GpuContext>,
    windows: HashMap<WindowId, WindowState>,
    controller: Option<Controller>,

    initial_emulator: Option<EmulatorHandle>,
}

impl Application {
    pub fn new(_title: String, initial_emulator: Option<EmulatorHandle>, show_logs: bool) -> Self {
        Self {
            show_logs,
            keypad_tracker: KeypadTracker::new(),
            modifiers: ModifiersState::empty(),
            gpu: None,
            windows: HashMap::new(),
            controller: None,
            initial_emulator,
        }
    }

    fn build_running_content(&self, gpu: &GpuContext, surface: &WindowSurface, emulator: EmulatorHandle) -> ScreenContent {
        let window_size = self.windows.values().next().map(|s| s.window.inner_size());
        let (window_width, window_height) = window_size.map(|s| (s.width, s.height)).unwrap_or((1, 1));
        let renderer = GbaRenderer::new(
            gpu,
            surface.surface_format(),
            window_width,
            window_height,
            emulator.viewport_width,
            emulator.viewport_height,
        );
        let fps_timer = FrameTimer::new(emulator.fps);
        ScreenContent::Running(Box::new(RunningContent {
            emulator,
            renderer,
            last_frame: None,
            fps_timer,
        }))
    }

    fn load_rom(&mut self, window_id: WindowId, rom_path: String) {
        match emulator::spawn(rom_path.clone(), None, self.show_logs) {
            Ok(handle) => {
                let Some(gpu) = self.gpu.as_ref() else { return };
                let Some(state) = self.windows.get_mut(&window_id) else {
                    return;
                };

                let window_size = state.window.inner_size();
                let renderer = GbaRenderer::new(
                    gpu,
                    state.surface.surface_format(),
                    window_size.width,
                    window_size.height,
                    handle.viewport_width,
                    handle.viewport_height,
                );
                let fps_timer = FrameTimer::new(handle.fps);
                state.content = ScreenContent::Running(Box::new(RunningContent {
                    emulator: handle,
                    renderer,
                    last_frame: None,
                    fps_timer,
                }));

                let rom_name = std::path::Path::new(&rom_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&rom_path)
                    .to_string();
                state.window.set_title(&format!("{BASE_TITLE} - {rom_name}"));
            }
            Err(e) => tracing::error!("failed to load rom {rom_path}: {e}"),
        }
    }

    fn drain_and_render(&mut self, window_id: WindowId) {
        let (Some(gpu), Some(state)) = (self.gpu.as_ref(), self.windows.get_mut(&window_id)) else {
            return;
        };

        if let Some(running) = state.content.running_mut() {
            while let Ok(frame) = running.emulator.frames.try_recv() {
                running.last_frame = Some(frame);
                running.fps_timer.count_frame();
            }
            state.gui.fps_overlay_mut().set_fps(running.fps_timer.fps());
        }

        let output = match state.surface.acquire() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let size = state.window.inner_size();
                state.surface.resize(gpu, size);
                if let Some(running) = state.content.running_mut() {
                    running.renderer.resize(gpu, size.width, size.height);
                }
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

        state.gui.cleanup(&prepared);

        gpu.queue().submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn save_screenshot(&self) -> Result<(), DesktopError> {
        //TODO: when a screenshot is taken we need to give some type of overlay or visual notification that it happened
        let Some(state) = self.windows.values().next() else {
            return Ok(());
        };
        let Some(running) = state.content.running() else {
            return Ok(());
        };
        let Some(frame_buffer) = &running.last_frame else {
            tracing::warn!("screenshot requested before any frame has arrived");
            return Ok(());
        };

        let viewport_width = running.emulator.viewport_width as f32;
        let viewport_height = running.emulator.viewport_height as f32;

        let win_w = state.surface.config().width as f32;
        let win_h = state.surface.config().height as f32;
        let scale = (win_w / viewport_width).min(win_h / viewport_height);
        let out_w = (viewport_width * scale).round().max(1.0) as u32;
        let out_h = (viewport_height * scale).round().max(1.0) as u32;

        let src_w = viewport_width as u32;
        let src_h = viewport_height as u32;

        let mut rgba_buffer = Vec::with_capacity((out_w as usize) * (out_h as usize) * 4);
        for y in 0..out_h {
            let src_y = (((y as f32) / scale) as u32).min(src_h - 1);
            for x in 0..out_w {
                let src_x = (((x as f32) / scale) as u32).min(src_w - 1);
                let pixel = frame_buffer[(src_y * src_w + src_x) as usize];
                rgba_buffer.push(((pixel >> 16) & 0xFF) as u8);
                rgba_buffer.push(((pixel >> 8) & 0xFF) as u8);
                rgba_buffer.push((pixel & 0xFF) as u8);
                rgba_buffer.push(0xFF);
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let name = format!("screenshot-{timestamp}.png");

        image::save_buffer(&name, &rgba_buffer, out_w, out_h, image::ColorType::Rgba8)?;
        tracing::info!("wrote {name} ({out_w}x{out_h})");
        Ok(())
    }

    fn send_emulator_command(&self, command: EmulatorCommand) {
        let Some(state) = self.windows.values().next() else { return };
        let Some(running) = state.content.running() else { return };
        if let Err(e) = running.emulator.commands.send(command) {
            tracing::error!("emulator command dropped (thread gone?): {e}");
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
                if self.windows.values().any(|w| w.content.running().is_some()) {
                    for window_state in self.windows.values_mut() {
                        *window_state.gui.fps_overlay_mut().show_mut() ^= true;
                    }
                }
            }

            HotKey::Screenshot => {
                if let Err(e) = self.save_screenshot() {
                    tracing::error!("screenshot failed: {e}");
                }
            }
            HotKey::TogglePause => {
                if self.windows.values().any(|w| w.content.running().is_some()) {
                    self.send_emulator_command(EmulatorCommand::TogglePause);
                    for window_state in self.windows.values_mut() {
                        *window_state.gui.paused_overlay_mut().show_mut() ^= true;
                    }
                }
            }
            //TODO: add opt-in pause-on-minimize/unfocus config
            HotKey::ToggleMaxSpeed => self.send_emulator_command(EmulatorCommand::ToggleMaxSpeed),
            HotKey::Reset => {
                self.send_emulator_command(EmulatorCommand::Reset);
                if let Some(state) = self.windows.values_mut().next()
                    && let Some(running) = state.content.running_mut()
                {
                    running.last_frame = None;
                    running.fps_timer = FrameTimer::new(running.emulator.fps);
                }
            }
        }

        true
    }
}

impl ApplicationHandler for Application {
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

        let surface = WindowSurface::new(gpu, window.clone());
        let gui = Gui::new(gpu.device(), surface.surface_format(), &window);

        let content = match self.initial_emulator.take() {
            Some(emulator) => self.build_running_content(gpu, &surface, emulator),
            None => ScreenContent::Splash,
        };

        self.windows.insert(
            window.id(),
            WindowState {
                window,
                surface,
                gui,
                content,
            },
        );
        self.controller = Controller::new();
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
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::Resized(size) => {
                if let (Some(gpu), Some(state)) = (self.gpu.as_ref(), self.windows.get_mut(&window_id)) {
                    state.surface.resize(gpu, size);
                    if let Some(running) = state.content.running_mut() {
                        running.renderer.resize(gpu, size.width, size.height);
                    }
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(controller) = self.controller.as_mut() {
            let keypad = self
                .windows
                .values()
                .find_map(|s| s.content.running().map(|r| r.emulator.keypad.clone()));
            if let Some(keypad) = keypad {
                for (button, pressed) in controller.poll() {
                    self.keypad_tracker.handle_controller_button(button, pressed, &keypad);
                }
            }
        }

        for state in self.windows.values() {
            state.window.request_redraw();
        }
    }
}
