use crate::theme;
use egui::{CornerRadius, RichText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Files,
    Hosting,
    Chat,
    Browser,
}

pub fn show(ui: &mut egui::Ui, current: &mut Page) {
    egui::Frame::NONE
        .fill(theme::BG)
        .inner_margin(egui::Margin::symmetric(20, 10))
        .corner_radius(CornerRadius::same(4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() * 0.05);
                nav_btn(ui, current, Page::Files, "File Sharing", true);
                ui.add_space(20.0);
                nav_btn(ui, current, Page::Hosting, "Website Hosting", false);
                ui.add_space(20.0);
                nav_btn(ui, current, Page::Chat, "Secure Chat", true);
                ui.add_space(20.0);
                nav_btn(ui, current, Page::Browser, "Mixnet Browser", false);
                ui.add_space(ui.available_width() * 0.05);
            });
        });
}

fn nav_btn(ui: &mut egui::Ui, current: &mut Page, page: Page, label: &str, enabled: bool) {
    let selected = *current == page;
    let fill = if selected {
        theme::PRIMARY
    } else {
        theme::BG
    };
    let text = if enabled {
        theme::TEXT
    } else {
        theme::MUTED
    };
    let resp = ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).color(text).strong().size(14.0))
            .fill(fill)
            .corner_radius(CornerRadius::same(4)),
    );
    if resp.clicked() {
        *current = page;
    }
    if !enabled {
        resp.on_hover_text("Coming soon — deferred past MVP");
    }
}
