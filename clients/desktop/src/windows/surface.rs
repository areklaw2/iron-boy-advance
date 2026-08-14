use std::sync::Arc;

use getset::{CopyGetters, Getters};
use winit::{dpi::PhysicalSize, window::Window};

use crate::gpu::GpuContext;

#[derive(Getters, CopyGetters)]
pub struct WindowSurface {
    _window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    #[getset(get = "pub")]
    config: wgpu::SurfaceConfiguration,
    #[getset(get_copy = "pub")]
    surface_format: wgpu::TextureFormat,
}

impl WindowSurface {
    pub fn new(gpu: &GpuContext, window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let surface = gpu
            .instance()
            .create_surface(window.clone())
            .expect("failed to create wgpu surface");

        let caps = surface.get_capabilities(gpu.adapter());
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Bgra8UnormSrgb)
            .or_else(|| caps.formats.iter().copied().find(|f| f.is_srgb()))
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(gpu.device(), &config);

        Self {
            _window: window,
            surface,
            config,
            surface_format,
        }
    }

    pub fn resize(&mut self, gpu: &GpuContext, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        // Surface::configure panics past max_texture_dimension_2d (2048 on downlevel defaults).
        let max_dim = gpu.device().limits().max_texture_dimension_2d;
        self.config.width = size.width.min(max_dim);
        self.config.height = size.height.min(max_dim);
        self.surface.configure(gpu.device(), &self.config);
    }

    pub fn acquire(&self) -> wgpu::CurrentSurfaceTexture {
        self.surface.get_current_texture()
    }
}
