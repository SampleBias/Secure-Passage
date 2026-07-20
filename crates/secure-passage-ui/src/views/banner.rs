use crate::theme;
use egui::{RichText, Sense};

/// Native banner artwork aspect (SVG is 1000×224).
const BANNER_ASPECT: f32 = 1000.0 / 224.0;

pub fn show(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        let banner = egui::include_image!("../../../../assets/images/blob-8c69c17.svg");
        // Scale to the content width so the art tracks the window.
        let avail = ui.available_width();
        let mut height = (avail / BANNER_ASPECT).clamp(96.0, 280.0);
        let mut width = height * BANNER_ASPECT;
        if width > avail {
            width = avail;
            height = width / BANNER_ASPECT;
        }

        ui.add(
            egui::Image::new(banner)
                .fit_to_exact_size(egui::vec2(width, height))
                .maintain_aspect_ratio(true)
                .sense(Sense::hover()),
        );

        ui.add_space(8.0);
        ui.label(
            RichText::new("SECURE-PASSAGE")
                .font(theme::title_font())
                .color(theme::TEXT)
                .strong(),
        );
        ui.label(
            RichText::new(
                "Secure and anonymous file sharing, web hosting, web browsing, and chat via the Nym mixnet",
            )
            .font(theme::tagline_font())
            .italics()
            .color(theme::TEXT),
        );
        ui.add_space(6.0);
    });
    ui.separator();
}
