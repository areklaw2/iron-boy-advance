use std::sync::Arc;

use ironboyadvance::Viewport;
use winit::{dpi::PhysicalSize, window::Window};

use crate::video::{FrameRenderer, GpuContext, Gui, WindowSurface};

pub(crate) struct WindowState {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: WindowSurface,
    pub(crate) renderer: Option<FrameRenderer>,
    pub(crate) gui: Gui,
}

impl WindowState {
    pub(crate) fn new(gpu: &GpuContext, window: Arc<Window>) -> Self {
        let surface = WindowSurface::new(gpu, window.clone());
        let gui = Gui::new(gpu.device(), surface.surface_format(), &window);
        Self {
            window,
            surface,
            gui,
            renderer: None,
        }
    }

    pub(crate) fn attach_renderer(&mut self, gpu: &GpuContext, viewport: Viewport) {
        self.renderer = Some(FrameRenderer::new(
            gpu,
            self.surface.surface_format(),
            self.surface.config().width,
            self.surface.config().height,
            viewport,
        ))
    }

    pub(crate) fn resize(&mut self, gpu: &GpuContext, size: PhysicalSize<u32>) {
        self.surface.resize(gpu, size);
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(gpu, self.surface.config().width, self.surface.config().height);
        }
    }
}
