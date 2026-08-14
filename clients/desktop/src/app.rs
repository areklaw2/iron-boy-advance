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
    DesktopError,
    controller::Controller,
    emulator::{EmulatorCommand, EmulatorHandle},
    frame::FrameTimer,
    gpu::GpuContext,
    gui::Gui,
    input::{HotKey, KeypadTracker, keycode_to_button, keycode_to_hotkey},
    renderer::Renderer,
};

struct WindowState {
    window: Arc<Window>,
    renderer: Renderer,
    gui: Gui,
}

pub struct Application {
    title: String,
    emulator: EmulatorHandle,
    keypad_tracker: KeypadTracker,
    modifiers: ModifiersState,

    gpu: Option<GpuContext>,
    windows: HashMap<WindowId, WindowState>,
    controller: Option<Controller>,

    last_frame: Option<Vec<u32>>,
    fps_timer: FrameTimer,
}

impl Application {
    pub fn new(title: String, emulator: EmulatorHandle) -> Self {
        let fps_timer = FrameTimer::new(emulator.fps);
        Self {
            title,
            emulator,
            keypad_tracker: KeypadTracker::new(),
            modifiers: ModifiersState::empty(),
            gpu: None,
            windows: HashMap::new(),
            controller: None,
            last_frame: None,
            fps_timer,
        }
    }

    fn drain_and_render(&mut self, window_id: WindowId) {
        let (Some(gpu), Some(state)) = (self.gpu.as_ref(), self.windows.get_mut(&window_id)) else {
            return;
        };

        while let Ok(frame) = self.emulator.frames.try_recv() {
            self.last_frame = Some(frame);
            self.fps_timer.count_frame();
        }

        state.gui.overlay_mut().set_fps(self.fps_timer.fps());

        let output = match state.renderer.acquire() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let size = state.window.inner_size();
                state.renderer.resize(gpu, size);
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Validation => {
                tracing::error!("wgpu surface validation error during acquire");
                return;
            }
        };

        if let Some(ref fb) = self.last_frame {
            state.renderer.upload_frame(gpu, fb);
        }

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("desktop-encoder"),
        });

        let screen = ScreenDescriptor {
            size_in_pixels: [state.renderer.config().width, state.renderer.config().height],
            pixels_per_point: state.window.scale_factor() as f32,
        };

        // Egui must upload its textures and vertex/index buffers BEFORE the render pass
        // begins, because begin_render_pass borrows the encoder.
        let prepared = state
            .gui
            .prepare(&state.window, gpu.device(), gpu.queue(), &mut encoder, &screen);

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

            if self.last_frame.is_some() {
                state.renderer.draw_frame(&mut rpass);
            }

            state.gui.paint(&mut rpass, &prepared, &screen);
        }

        state.gui.cleanup(&prepared);

        gpu.queue().submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn save_screenshot(&self) -> Result<(), DesktopError> {
        //TODO: when a screenshot is taken we need to give some type of overlay or visual notification that it happened
        let Some(ref frame_buffer) = self.last_frame else {
            tracing::warn!("screenshot requested before any frame has arrived");
            return Ok(());
        };

        let Some(state) = self.windows.values().next() else {
            return Ok(());
        };

        let viewport_width = self.emulator.viewport_width as f32;
        let viewport_height = self.emulator.viewport_height as f32;

        let win_w = state.renderer.config().width as f32;
        let win_h = state.renderer.config().height as f32;
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
        if let Err(e) = self.emulator.commands.send(command) {
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
            HotKey::ToggleFpsOverlay => {
                for window_state in self.windows.values_mut() {
                    *window_state.gui.overlay_mut().show_mut() ^= true;
                }
            }

            HotKey::Screenshot => {
                if let Err(e) = self.save_screenshot() {
                    tracing::error!("screenshot failed: {e}");
                }
            }
            HotKey::TogglePause => self.send_emulator_command(EmulatorCommand::TogglePause), //TODO: add an overlay for paused state
            //TODO: add opt-in pause-on-minimize/unfocus config
            HotKey::ToggleMaxSpeed => self.send_emulator_command(EmulatorCommand::ToggleMaxSpeed),
            HotKey::Reset => {
                self.send_emulator_command(EmulatorCommand::Reset);
                self.last_frame = None;
                self.fps_timer = FrameTimer::new(self.emulator.fps);
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

        let attrs = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(LogicalSize::new(
                self.emulator.viewport_width as u32 * 6,
                self.emulator.viewport_height as u32 * 6,
            ));
        let window = Arc::new(event_loop.create_window(attrs).expect("failed to create window"));

        if self.gpu.is_none() {
            self.gpu = Some(pollster::block_on(GpuContext::new()));
        }
        let gpu = self.gpu.as_ref().unwrap();

        let renderer = Renderer::new(
            gpu,
            window.clone(),
            self.emulator.viewport_width,
            self.emulator.viewport_height,
        );
        let gui = Gui::new(gpu.device(), renderer.surface_format(), &window);

        self.windows.insert(window.id(), WindowState { window, renderer, gui });
        self.controller = Controller::new();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(state) = self.windows.get_mut(&window_id) else {
            return;
        };
        let window = state.window.clone();

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
                    state.renderer.resize(gpu, size);
                }
                window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                if code == KeyCode::Escape && state == ElementState::Pressed {
                    event_loop.exit();
                    return;
                }

                if self.handle_hotkey(code, state) {
                    return;
                }

                if !egui_consumed && let Some(button) = keycode_to_button(code) {
                    self.keypad_tracker.handle_keyboard_button(
                        button,
                        state == ElementState::Pressed,
                        &self.emulator.keypad,
                    );
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
            for (button, pressed) in controller.poll() {
                self.keypad_tracker
                    .handle_controller_button(button, pressed, &self.emulator.keypad);
            }
        }

        for state in self.windows.values() {
            state.window.request_redraw();
        }
    }
}
