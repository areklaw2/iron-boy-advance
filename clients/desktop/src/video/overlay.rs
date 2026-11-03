use egui::{Align2, Area, Color32, FontFamily, Id, Label, RichText, TextWrapMode, vec2};

pub(crate) struct Overlays {
    pub(crate) fps: Option<f64>,
    pub(crate) paused: bool,
}

pub(super) enum OverlayKind {
    Fps(f64),
    Paused,
}

impl OverlayKind {
    pub(super) fn draw(&self, ctx: &egui::Context) {
        let (anchor, offset) = self.anchor();
        Area::new(Id::unique(self.id()))
            .anchor(anchor, offset)
            .interactable(false)
            .show(ctx, |ui| {
                ui.add(
                    Label::new(
                        RichText::new(self.text())
                            .color(self.color())
                            .size(self.size())
                            .family(FontFamily::Name("gbboot".into())),
                    )
                    .wrap_mode(TextWrapMode::Extend),
                );
            });
    }

    fn id(&self) -> &'static str {
        match self {
            OverlayKind::Fps(_) => "fps_overlay",
            OverlayKind::Paused => "paused_overlay",
        }
    }

    fn anchor(&self) -> (Align2, egui::Vec2) {
        match self {
            OverlayKind::Fps(_) => (Align2::RIGHT_BOTTOM, vec2(-10.0, -10.0)),
            OverlayKind::Paused => (Align2::CENTER_CENTER, vec2(0.0, 0.0)),
        }
    }

    fn text(&self) -> String {
        match self {
            OverlayKind::Fps(fps) => format!("{fps:.1} FPS"),
            OverlayKind::Paused => "PAUSED".to_owned(),
        }
    }

    fn color(&self) -> Color32 {
        match self {
            OverlayKind::Fps(_) => Color32::GREEN,
            OverlayKind::Paused => Color32::YELLOW,
        }
    }

    fn size(&self) -> f32 {
        match self {
            OverlayKind::Fps(_) => 24.0,
            OverlayKind::Paused => 72.0,
        }
    }
}
