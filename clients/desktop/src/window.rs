use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    BASE_TITLE,
    emulator::EmulatorHandle,
    frame::FrameTimer,
    video::{FrameRenderer, GpuContext, Gui, WindowSurface},
};

pub struct RunningContent {
    pub emulator: EmulatorHandle,
    pub renderer: FrameRenderer,
    pub last_frame: Option<Vec<u32>>,
    pub fps_timer: FrameTimer,
    pub paused: bool,
    pub max_speed: bool,
}

impl RunningContent {
    pub fn new(gpu: &GpuContext, surface: &WindowSurface, emulator: EmulatorHandle) -> Self {
        let renderer = FrameRenderer::new(
            gpu,
            surface.surface_format(),
            surface.config().width,
            surface.config().height,
            emulator.viewport_width,
            emulator.viewport_height,
        );
        let fps_timer = FrameTimer::new(emulator.fps);
        Self {
            emulator,
            renderer,
            last_frame: None,
            fps_timer,
            paused: false,
            max_speed: false,
        }
    }
}

pub enum ScreenContent {
    Splash,
    Running(Box<RunningContent>),
}

impl ScreenContent {
    pub fn running(&self) -> Option<&RunningContent> {
        match self {
            ScreenContent::Running(running) => Some(running),
            ScreenContent::Splash => None,
        }
    }

    pub fn running_mut(&mut self) -> Option<&mut RunningContent> {
        match self {
            ScreenContent::Running(running) => Some(running),
            ScreenContent::Splash => None,
        }
    }
}

pub(crate) struct WindowState {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: WindowSurface,
    pub(crate) gui: Gui,
    pub(crate) content: ScreenContent,
}

impl WindowState {
    pub fn new(gpu: &GpuContext, window: Arc<Window>, emulator: Option<EmulatorHandle>) -> Self {
        let surface = WindowSurface::new(gpu, window.clone());
        let gui = Gui::new(gpu.device(), surface.surface_format(), &window);
        let content = match emulator {
            Some(emulator) => ScreenContent::Running(Box::new(RunningContent::new(gpu, &surface, emulator))),
            None => ScreenContent::Splash,
        };
        Self {
            window,
            surface,
            gui,
            content,
        }
    }

    pub fn start_running(&mut self, gpu: &GpuContext, emulator: EmulatorHandle, rom_name: &str) {
        self.content = ScreenContent::Running(Box::new(RunningContent::new(gpu, &self.surface, emulator)));
        self.window.set_title(&format!("{BASE_TITLE} - {rom_name}"));
    }

    pub fn resize(&mut self, gpu: &GpuContext, size: PhysicalSize<u32>) {
        self.surface.resize(gpu, size);
        if let Some(running) = self.content.running_mut() {
            running.renderer.resize(gpu, size.width, size.height);
        }
    }

    pub fn drain_frames(&mut self) {
        if let Some(running) = self.content.running_mut() {
            while let Ok(frame) = running.emulator.frames.try_recv() {
                running.last_frame = Some(frame);
                running.fps_timer.count_frame();
            }
            self.gui.fps_overlay_mut().set_fps(running.fps_timer.fps());
            self.gui.paused_overlay_mut().set_show(running.paused);
        }
    }
}
