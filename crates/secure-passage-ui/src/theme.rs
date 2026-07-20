//! Visual theme matching the original Secure Passage PyQt StyleManager.

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, Style, Theme, Visuals};

pub const BG: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const PANEL: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3A);
pub const INPUT: Color32 = Color32::from_rgb(0x44, 0x44, 0x44);
pub const PRIMARY: Color32 = Color32::from_rgb(0xA6, 0x8C, 0xFF);
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0xB7, 0x9D, 0xFF);
pub const SUCCESS: Color32 = Color32::from_rgb(0x4C, 0xAF, 0x50);
pub const DANGER: Color32 = Color32::from_rgb(0xF4, 0x43, 0x36);
pub const SECONDARY: Color32 = Color32::from_rgb(0x55, 0x55, 0x55);
pub const TEXT: Color32 = Color32::WHITE;
pub const MUTED: Color32 = Color32::from_rgb(0xAA, 0xAA, 0xAA);
pub const ORANGE: Color32 = Color32::from_rgb(0xFF, 0xA5, 0x00);

pub fn apply(ctx: &egui::Context) {
    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.visuals = dark_visuals();
    ctx.set_style_of(Theme::Dark, style);
    ctx.set_theme(Theme::Dark);

    // Prefer egui's built-in monospace faces globally (IBM Plex–like look).
    // Do not insert a literal "monospace" name — that has no font data and panics.
    let mut fonts = egui::FontDefinitions::default();
    if let Some(mono) = fonts.families.get(&FontFamily::Monospace).cloned() {
        fonts.families.insert(FontFamily::Proportional, mono);
    }
    ctx.set_fonts(fonts);
}

fn dark_visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.override_text_color = Some(TEXT);
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.extreme_bg_color = INPUT;
    v.widgets.inactive.bg_fill = SECONDARY;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.bg_fill = PRIMARY_HOVER;
    v.widgets.active.bg_fill = PRIMARY;
    v.selection.bg_fill = PRIMARY;
    v.widgets.noninteractive.bg_fill = PANEL;
    v.window_corner_radius = CornerRadius::same(5);
    v.menu_corner_radius = CornerRadius::same(4);
    v
}

pub fn title_font() -> FontId {
    FontId::new(24.0, FontFamily::Monospace)
}

#[allow(dead_code)]
pub fn body_font() -> FontId {
    FontId::new(14.0, FontFamily::Monospace)
}

pub fn tagline_font() -> FontId {
    FontId::new(13.0, FontFamily::Monospace)
}

pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(TEXT).strong())
            .fill(PRIMARY)
            .corner_radius(CornerRadius::same(4)),
    )
}

pub fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(TEXT).strong())
            .fill(SECONDARY)
            .corner_radius(CornerRadius::same(4)),
    )
}

pub fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(TEXT).strong())
            .fill(DANGER)
            .corner_radius(CornerRadius::same(4)),
    )
}

pub fn success_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(TEXT).strong())
            .fill(SUCCESS)
            .corner_radius(CornerRadius::same(4)),
    )
}
