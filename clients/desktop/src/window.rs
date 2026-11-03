use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use ironboyadvance::Viewport;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    emulator::Emulator,
    video::{EguiLayer, FrameRenderer, GpuContext, Overlays, WindowSurface},
};

pub(crate) struct WindowState {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: WindowSurface,
    pub(crate) frame_renderer: Option<FrameRenderer>,
    pub(crate) egui_layer: EguiLayer,
}

impl WindowState {
    pub(crate) fn new(gpu: &GpuContext, window: Arc<Window>) -> Self {
        let surface = WindowSurface::new(gpu, window.clone());
        let egui_layer = EguiLayer::new(gpu.device(), surface.surface_format(), &window);
        Self {
            window,
            surface,
            egui_layer,
            frame_renderer: None,
        }
    }

    pub(crate) fn attach_renderer(&mut self, gpu: &GpuContext, viewport: Viewport) {
        self.frame_renderer = Some(FrameRenderer::new(
            gpu,
            self.surface.surface_format(),
            self.surface.config().width,
            self.surface.config().height,
            viewport,
        ))
    }

    pub(crate) fn resize(&mut self, gpu: &GpuContext, size: PhysicalSize<u32>) {
        self.surface.resize(gpu, size);
        if let Some(frame_renderer) = &mut self.frame_renderer {
            frame_renderer.resize(gpu, self.surface.config().width, self.surface.config().height);
        }
    }

    pub(crate) fn render(&mut self, gpu: &GpuContext, emulator: Option<&Emulator>, overlays: &Overlays) {
        let output = match self.surface.acquire() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.resize(gpu, self.window.inner_size());
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Validation => {
                tracing::error!("wgpu surface validation error during acquire");
                return;
            }
        };

        let frame = emulator.and_then(Emulator::last_frame);
        if let (Some(frame_renderer), Some(frame)) = (self.frame_renderer.as_ref(), frame) {
            frame_renderer.upload_frame(gpu, frame);
        }

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("desktop-encoder"),
        });
        let screen = ScreenDescriptor {
            size_in_pixels: [self.surface.config().width, self.surface.config().height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let prepared = self
            .egui_layer
            .prepare(&self.window, gpu, &mut encoder, &screen, overlays, emulator.is_none());

        {
            let mut pass = encoder
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

            if let (Some(frame_renderer), Some(_)) = (self.frame_renderer.as_ref(), frame) {
                frame_renderer.draw_frame(&mut pass);
            }
            self.egui_layer.paint(&mut pass, &prepared, &screen);
        }

        self.egui_layer.cleanup(prepared);
        gpu.queue().submit(std::iter::once(encoder.finish()));
        gpu.queue().present(output);
    }
}
