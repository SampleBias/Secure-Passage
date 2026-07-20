use crate::theme;
use egui::{RichText, Sense};

pub fn show(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        // Banner image (SVG ship) when available
        let banner = egui::include_image!("../../../../assets/images/blob-8c69c17.svg");
        ui.add(
            egui::Image::new(banner)
                .fit_to_exact_size(egui::vec2(ui.available_width().min(720.0), 120.0))
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
