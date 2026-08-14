use egui::{Color32, FontFamily, RichText};

pub fn draw_splash(ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 2.0 - 40.0);
            ui.label(
                RichText::new("Iron Boy Advance")
                    .color(Color32::WHITE)
                    .size(48.0)
                    .family(FontFamily::Name("gbboot".into())),
            );
            ui.add_space(12.0);
            ui.label(
                RichText::new("Drag a ROM here to begin")
                    .color(Color32::GRAY)
                    .size(20.0)
                    .family(FontFamily::Name("gbboot".into())),
            );
        });
    });
}
