use crate::theme;
use egui::RichText;

pub struct StatusBarState {
    pub mixnet: String,
    pub vpn_label: String,
    pub vpn_ok: bool,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            mixnet: "Nym not connected".into(),
            vpn_label: "VPN: Unknown".into(),
            vpn_ok: false,
        }
    }
}

pub enum StatusAction {
    None,
    Logs,
    Kill,
    VpnLink,
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
            if theme::success_button(ui, "Need a VPN that will protect you?").clicked() {
                action = StatusAction::VpnLink;
            }
            let vpn_color = if state.vpn_ok {
                theme::SUCCESS
            } else {
                theme::ORANGE
            };
            ui.label(RichText::new(&state.vpn_label).color(vpn_color).size(12.0));
        });
    });
    action
}
