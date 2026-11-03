use egui::{FontData, FontDefinitions, FontFamily, TexturesDelta, ViewportId, epaint::ClippedPrimitive};
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use winit::{event::WindowEvent, window::Window};

use crate::video::{
    GpuContext,
    overlay::{OverlayKind, Overlays},
    splash::draw_splash,
};

pub(crate) struct EguiLayer {
    context: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

pub(crate) struct PreparedFrame {
    pub paint_jobs: Vec<ClippedPrimitive>,
    pub textures_delta: TexturesDelta,
}

impl EguiLayer {
    pub(crate) fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, window: &Window) -> Self {
        let context = egui::Context::default();
        install_gbboot_font(&context);
        let state = egui_winit::State::new(
            context.clone(),
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: false,
                predictable_texture_filtering: false,
            },
        );
        Self {
            context,
            state,
            renderer,
        }
    }

    pub(crate) fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    pub(crate) fn prepare(
        &mut self,
        window: &Window,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        screen: &ScreenDescriptor,
        overlays: &Overlays,
        show_splash: bool,
    ) -> PreparedFrame {
        let raw_input = self.state.take_egui_input(window);
        let mut full_output = self.context.run_ui(raw_input, |ui| {
            if show_splash {
                draw_splash(ui);
            }

            if let Some(fps) = overlays.fps {
                OverlayKind::Fps(fps).draw(ui.ctx());
            }
            if overlays.paused {
                OverlayKind::Paused.draw(ui.ctx());
            }
        });

        self.state.handle_platform_output(window, full_output.platform_output);

        let paint_jobs = self.context.tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, deltas) in full_output.textures_delta.set.drain() {
            for delta in deltas {
                self.renderer.update_texture(gpu.device(), gpu.queue(), id, &delta);
            }
        }
        self.renderer
            .update_buffers(gpu.device(), gpu.queue(), encoder, &paint_jobs, screen);

        PreparedFrame {
            paint_jobs,
            textures_delta: full_output.textures_delta,
        }
    }

    pub(crate) fn paint(&self, rpass: &mut wgpu::RenderPass<'static>, prepared: &PreparedFrame, screen: &ScreenDescriptor) {
        self.renderer.render(rpass, &prepared.paint_jobs, screen);
    }

    pub(crate) fn cleanup(&mut self, mut prepared: PreparedFrame) {
        for id in prepared.textures_delta.free.drain() {
            self.renderer.free_texture(&id);
        }
    }
}

fn install_gbboot_font(ctx: &egui::Context) {
    const GBBOOT_TTF: &[u8] = include_bytes!("../../../../media/gbboot-alpm.ttf");
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("gbboot".to_owned(), FontData::from_static(GBBOOT_TTF).into());
    fonts
        .families
        .insert(FontFamily::Name("gbboot".into()), vec!["gbboot".to_owned()]);
    ctx.set_fonts(fonts);
}
