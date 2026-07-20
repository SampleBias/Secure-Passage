//! Visual theme matching the original Secure Passage PyQt StyleManager.

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, Style, Theme, Visuals};

pub const BG: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const PANEL: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3A);
pub const INPUT: Color32 = Color32::from_rgb(0x44, 0x44, 0x44);
/// Neon mint primary (sampled from brand CTA).
pub const PRIMARY: Color32 = Color32::from_rgb(0x07, 0xFF, 0x94);
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0x4D, 0xFF, 0xB0);
pub const PRIMARY_ACTIVE: Color32 = Color32::from_rgb(0x06, 0xD9, 0x7E);
/// Dark text for contrast on the bright primary fill.
pub const ON_PRIMARY: Color32 = Color32::from_rgb(0x12, 0x12, 0x12);
pub const DANGER: Color32 = Color32::from_rgb(0xF4, 0x43, 0x36);
pub const DANGER_HOVER: Color32 = Color32::from_rgb(0xFF, 0x6B, 0x5E);
pub const SECONDARY: Color32 = Color32::from_rgb(0x55, 0x55, 0x55);
pub const SECONDARY_HOVER: Color32 = Color32::from_rgb(0x6A, 0x6A, 0x6A);
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
    // Do not set override_text_color — it forces every galley white and
    // defeats selection/hover text colors on green fills.
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.extreme_bg_color = INPUT;
    v.widgets.noninteractive.bg_fill = PANEL;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.bg_fill = SECONDARY;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.bg_fill = PRIMARY_HOVER;
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, ON_PRIMARY);
    v.widgets.active.bg_fill = PRIMARY_ACTIVE;
    v.widgets.active.fg_stroke = Stroke::new(1.0, ON_PRIMARY);
    v.selection.bg_fill = PRIMARY;
    // Dark text on neon green selection (e.g. Send/Host · Receive tabs).
    v.selection.stroke = Stroke::new(1.0, ON_PRIMARY);
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

/// Previous-frame hover/press so fill can update before the button is painted.
fn fill_for(ui: &egui::Ui, base: Color32, hover: Color32, active: Color32) -> Color32 {
    let id = ui.next_auto_id();
    match ui.ctx().read_response(id) {
        Some(r) if r.is_pointer_button_down_on() => active,
        Some(r) if r.hovered() => hover,
        _ => base,
    }
}

/// Primary fill for custom `egui::Button`s (uses previous-frame hover state).
pub fn primary_fill(ui: &egui::Ui) -> Color32 {
    fill_for(ui, PRIMARY, PRIMARY_HOVER, PRIMARY_ACTIVE)
}

/// Mode tab (Send/Host · Receive, Host chat · Join chat).
/// Selected/hovered green fills always use dark text for contrast.
pub fn mode_tab(ui: &mut egui::Ui, selected: bool, label: &str) -> egui::Response {
    let text = if selected {
        egui::RichText::new(label).color(ON_PRIMARY).strong()
    } else {
        egui::RichText::new(label).color(TEXT)
    };
    ui.selectable_label(selected, text)
}

pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let fill = primary_fill(ui);
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(ON_PRIMARY).strong())
            .fill(fill)
            .corner_radius(CornerRadius::same(4)),
    )
}

pub fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let fill = fill_for(ui, SECONDARY, SECONDARY_HOVER, SECONDARY);
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(TEXT).strong())
            .fill(fill)
            .corner_radius(CornerRadius::same(4)),
    )
}

pub fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let fill = fill_for(ui, DANGER, DANGER_HOVER, DANGER);
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(TEXT).strong())
            .fill(fill)
            .corner_radius(CornerRadius::same(4)),
    )
}

/// "Enter App" — checkered green when ready, flat grey when not.
pub fn enter_app_button(ui: &mut egui::Ui, ready: bool) -> egui::Response {
    let label = "Enter App";
    let padding = ui.spacing().button_padding;
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font_id.clone(), TEXT);
    let desired = galley.size() + 2.0 * padding;

    let sense = if ready {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(desired, sense);
    let rounding = CornerRadius::same(4);

    if ui.is_rect_visible(rect) {
        if ready {
            let (c1, c2) = if response.is_pointer_button_down_on() {
                (PRIMARY_ACTIVE, Color32::from_rgb(0x05, 0xB8, 0x6C))
            } else if response.hovered() {
                (PRIMARY_HOVER, Color32::from_rgb(0x2A, 0xFF, 0xA0))
            } else {
                (PRIMARY, PRIMARY_ACTIVE)
            };
            paint_checkerboard(ui.painter(), rect, rounding, c1, c2, 7.0);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                font_id,
                ON_PRIMARY,
            );
        } else {
            ui.painter().rect_filled(rect, rounding, SECONDARY);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                font_id,
                MUTED,
            );
        }
    }

    response
}

fn paint_checkerboard(
    painter: &egui::Painter,
    rect: egui::Rect,
    rounding: CornerRadius,
    c1: Color32,
    c2: Color32,
    cell: f32,
) {
    // Base fill so rounded corners stay solid.
    painter.rect_filled(rect, rounding, c1);
    let painter = painter.with_clip_rect(rect);
    let mut row = 0i32;
    let mut y = rect.top();
    while y < rect.bottom() {
        let mut col = 0i32;
        let mut x = rect.left();
        while x < rect.right() {
            if (row + col) % 2 != 0 {
                painter.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell, cell)),
                    0.0,
                    c2,
                );
            }
            x += cell;
            col += 1;
        }
        y += cell;
        row += 1;
    }
}
