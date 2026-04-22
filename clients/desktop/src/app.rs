use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use ironboyadvance_core::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    DesktopError,
    emulator::{EmulatorCommand, EmulatorHandle},
    frame::FrameTimer,
    gui::Gui,
    input::{HotKey, KeypadTracker, keycode_to_hotkey},
    renderer::Renderer,
};

pub struct Application {
    title: String,
    emulator: EmulatorHandle,
    keypad_tracker: KeypadTracker,

    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    gui: Option<Gui>,

    last_frame: Option<Vec<u32>>,
    fps_timer: FrameTimer,
}

impl Application {
    pub fn new(title: String, emulator: EmulatorHandle) -> Self {
        Self {
            title,
            emulator,
            keypad_tracker: KeypadTracker::new(),
            window: None,
            renderer: None,
            gui: None,
            last_frame: None,
            fps_timer: FrameTimer::new(),
        }
    }

    fn drain_and_render(&mut self) {
        let (Some(window), Some(renderer), Some(gui)) = (self.window.as_ref(), self.renderer.as_mut(), self.gui.as_mut())
        else {
            return;
        };

        while let Ok(frame) = self.emulator.frames.try_recv() {
            self.last_frame = Some(frame);
            self.fps_timer.count_frame();
        }

        if let Some(ref fb) = self.last_frame {
            renderer.upload_frame(fb);
        }

        gui.overlay_mut().set_fps(self.fps_timer.fps());

        let output = match renderer.acquire() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                renderer.resize(window.inner_size());
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Validation => {
                tracing::error!("wgpu surface validation error during acquire");
                return;
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("desktop-encoder"),
        });

        let screen = ScreenDescriptor {
            size_in_pixels: [renderer.config().width, renderer.config().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        // Egui must upload its textures and vertex/index buffers BEFORE the render pass
        // begins, because begin_render_pass borrows the encoder.
        let prepared = gui.prepare(window, renderer.device(), renderer.queue(), &mut encoder, &screen);

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
                renderer.draw_frame(&mut rpass);
            }

            gui.paint(&mut rpass, &prepared, &screen);
        }

        gui.cleanup(&prepared);

        renderer.queue().submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn save_screenshot(&self) -> Result<(), DesktopError> {
        let Some(ref frame_buffer) = self.last_frame else {
            tracing::warn!("screenshot requested before any frame has arrived");
            return Ok(());
        };

        let Some(renderer) = self.renderer.as_ref() else {
            return Ok(());
        };

        let win_w = renderer.config().width as f32;
        let win_h = renderer.config().height as f32;
        let scale = (win_w / VIEWPORT_WIDTH as f32).min(win_h / VIEWPORT_HEIGHT as f32);
        let out_w = (VIEWPORT_WIDTH as f32 * scale).round().max(1.0) as u32;
        let out_h = (VIEWPORT_HEIGHT as f32 * scale).round().max(1.0) as u32;

        let src_w = VIEWPORT_WIDTH as u32;
        let src_h = VIEWPORT_HEIGHT as u32;

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

        let Some(hotkey) = keycode_to_hotkey(code) else {
            return false;
        };

        match hotkey {
            HotKey::ToggleFpsOverlay => {
                if let Some(gui) = self.gui.as_mut() {
                    *gui.overlay_mut().show_mut() ^= true;
                }
            }

            HotKey::Screenshot => {
                if let Err(e) = self.save_screenshot() {
                    tracing::error!("screenshot failed: {e}");
                }
            }
            HotKey::TogglePause => self.send_emulator_command(EmulatorCommand::TogglePause),
            HotKey::ToggleMaxSpeed => self.send_emulator_command(EmulatorCommand::ToggleMaxSpeed),
        }

        true
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(LogicalSize::new(VIEWPORT_WIDTH as u32 * 6, VIEWPORT_HEIGHT as u32 * 6));
        let window = Arc::new(event_loop.create_window(attrs).expect("failed to create window"));

        let renderer = pollster::block_on(Renderer::new(window.clone()));
        let gui = Gui::new(renderer.device(), renderer.surface_format(), &window);

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.gui = Some(gui);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.clone() else {
            return;
        };

        let egui_consumed = self.gui.as_mut().map(|g| g.on_window_event(&window, &event)).unwrap_or(false);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.resize(size);
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

                if !egui_consumed {
                    self.keypad_tracker.handle_button(code, state, &self.emulator.keypad);
                }
            }
            WindowEvent::RedrawRequested => {
                self.drain_and_render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
