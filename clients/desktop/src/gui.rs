use egui::{
    Align2, Area, Color32, FontData, FontDefinitions, FontFamily, Id, Label, RichText, TextWrapMode, TexturesDelta,
    ViewportId, epaint::ClippedPrimitive, vec2,
};
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use getset::{MutGetters, Setters};
use winit::{event::WindowEvent, window::Window};

#[derive(Setters, MutGetters)]
pub struct Overlay {
    #[getset(set = "pub")]
    fps: f64,
    #[getset(get_mut = "pub")]
    show: bool,
}

impl Overlay {
    pub fn new() -> Self {
        Self { fps: 0.0, show: false }
    }
}

#[derive(MutGetters)]
pub struct Gui {
    context: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    #[getset(get_mut = "pub")]
    overlay: Overlay,
}

/// Artifacts produced by `Gui::prepare` that must live until after painting.
pub struct PreparedFrame {
    pub paint_jobs: Vec<ClippedPrimitive>,
    pub textures_delta: TexturesDelta,
}

impl Gui {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, window: &Window) -> Self {
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
            overlay: Overlay::new(),
        }
    }

    /// Returns true if egui wants to consume this event (so emu input should skip it).
    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    /// Step 1: build the egui frame and upload textures + vertex/index buffers.
    /// Must be called BEFORE `begin_render_pass` on the encoder.
    pub fn prepare(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        screen: &ScreenDescriptor,
    ) -> PreparedFrame {
        let raw_input = self.state.take_egui_input(window);
        let overlay = &self.overlay;
        let full_output = self.context.run_ui(raw_input, |ui| {
            draw_overlay(ui.ctx(), overlay);
        });

        self.state.handle_platform_output(window, full_output.platform_output);

        let paint_jobs = self.context.tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, delta);
        }
        self.renderer.update_buffers(device, queue, encoder, &paint_jobs, screen);

        PreparedFrame {
            paint_jobs,
            textures_delta: full_output.textures_delta,
        }
    }

    /// Step 2: paint egui into a render pass (detached from the encoder).
    pub fn paint(&self, rpass: &mut wgpu::RenderPass<'static>, prepared: &PreparedFrame, screen: &ScreenDescriptor) {
        self.renderer.render(rpass, &prepared.paint_jobs, screen);
    }

    /// Step 3: free textures egui is done with. Call after the pass has ended.
    pub fn cleanup(&mut self, prepared: &PreparedFrame) {
        for id in &prepared.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}

fn draw_overlay(ctx: &egui::Context, overlay: &Overlay) {
    if !overlay.show {
        return;
    }

    Area::new(Id::new("fps_overlay"))
        .anchor(Align2::RIGHT_BOTTOM, vec2(-10.0, -10.0))
        .interactable(false)
        .show(ctx, |ui| {
            ui.add(
                Label::new(
                    RichText::new(format!("{:.1} FPS", overlay.fps))
                        .color(Color32::GREEN)
                        .size(24.0)
                        .family(FontFamily::Name("gbboot".into())),
                )
                .wrap_mode(TextWrapMode::Extend),
            );
        });
}

fn install_gbboot_font(ctx: &egui::Context) {
    const GBBOOT_TTF: &[u8] = include_bytes!("../../../media/gbboot-alpm.ttf");
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("gbboot".to_owned(), FontData::from_static(GBBOOT_TTF).into());
    fonts
        .families
        .insert(FontFamily::Name("gbboot".into()), vec!["gbboot".to_owned()]);
    ctx.set_fonts(fonts);
}
