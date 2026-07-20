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
                nav_btn(ui, current, Page::Hosting, "Website Hosting (in dev)", false);
                ui.add_space(20.0);
                nav_btn(ui, current, Page::Chat, "Secure Chat", true);
                ui.add_space(20.0);
                nav_btn(ui, current, Page::Browser, "Nym Browser (in dev)", false);
                ui.add_space(ui.available_width() * 0.05);
            });
        });
}

fn nav_btn(ui: &mut egui::Ui, current: &mut Page, page: Page, label: &str, enabled: bool) {
    let selected = *current == page;
    let id = ui.next_auto_id();
    let hovered = ui
        .ctx()
        .read_response(id)
        .is_some_and(|r| r.hovered() || r.is_pointer_button_down_on());

    let (fill, text) = if selected {
        (theme::PRIMARY, theme::ON_PRIMARY)
    } else if enabled && hovered {
        (theme::PRIMARY_HOVER, theme::ON_PRIMARY)
    } else if enabled {
        (theme::BG, theme::TEXT)
    } else {
        (theme::BG, theme::MUTED)
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
