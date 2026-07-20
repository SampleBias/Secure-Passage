use crate::theme;
use egui::RichText;

pub struct StatusBarState {
    pub mixnet: String,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            mixnet: "Nym not connected".into(),
        }
    }
}

pub enum StatusAction {
    None,
    Logs,
    Kill,
}

pub fn show(ui: &mut egui::Ui, state: &StatusBarState) -> StatusAction {
    let mut action = StatusAction::None;
    ui.horizontal(|ui| {
        ui.label(RichText::new(&state.mixnet).color(theme::TEXT).size(12.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if theme::danger_button(ui, "Kill").clicked() {
                action = StatusAction::Kill;
            }
            if theme::primary_button(ui, "Logs").clicked() {
                action = StatusAction::Logs;
            }
        });
    });
    action
}
