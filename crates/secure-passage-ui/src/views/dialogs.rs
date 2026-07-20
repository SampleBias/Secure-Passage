use crate::theme;
use egui::{ProgressBar, RichText};

pub struct VpnDialogState {
    pub checking: bool,
    pub detail: String,
    pub detected: Option<bool>,
    pub done: bool,
    pub skipped: bool,
}

impl Default for VpnDialogState {
    fn default() -> Self {
        Self {
            checking: false,
            detail: "Click Check to look up your public IP / org.".into(),
            detected: None,
            done: false,
            skipped: false,
        }
    }
}

pub fn show_vpn(ui: &mut egui::Ui, state: &mut VpnDialogState) -> VpnAction {
    let mut action = VpnAction::None;
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);
        ui.label(
            RichText::new("VPN Check")
                .font(theme::title_font())
                .color(theme::TEXT),
        );
        ui.add_space(12.0);
        ui.label(
            RichText::new(
                "A VPN alongside the Nym mixnet can add an extra hop of protection. This check is optional and best-effort.",
            )
            .color(theme::MUTED),
        );
        ui.add_space(20.0);

        if state.checking {
            ui.spinner();
            ui.label("Checking public IP…");
        } else {
            ui.label(RichText::new(&state.detail).color(theme::TEXT));
            if let Some(detected) = state.detected {
                let (label, color) = if detected {
                    ("VPN / hosting likely detected", theme::SUCCESS)
                } else {
                    ("No clear VPN signature detected", theme::ORANGE)
                };
                ui.label(RichText::new(label).color(color).strong());
            }
        }

        ui.add_space(24.0);
        ui.horizontal(|ui| {
            if theme::secondary_button(ui, "Check").clicked() && !state.checking {
                action = VpnAction::Check;
            }
            if theme::primary_button(ui, "Continue").clicked() {
                state.done = true;
                action = VpnAction::Continue;
            }
            if theme::secondary_button(ui, "Skip").clicked() {
                state.skipped = true;
                state.done = true;
                action = VpnAction::Continue;
            }
            if theme::danger_button(ui, "Quit").clicked() {
                action = VpnAction::Quit;
            }
        });
    });
    action
}

pub enum VpnAction {
    None,
    Check,
    Continue,
    Quit,
}

pub struct NymConnectState {
    pub connecting: bool,
    pub percent: u8,
    pub message: String,
    pub error: Option<String>,
    pub connected: bool,
}

impl Default for NymConnectState {
    fn default() -> Self {
        Self {
            connecting: false,
            percent: 0,
            message: "Ready to connect to the Nym mixnet.".into(),
            error: None,
            connected: false,
        }
    }
}

pub enum NymConnectAction {
    None,
    Connect,
    Continue,
    Quit,
}

pub fn show_nym_connect(ui: &mut egui::Ui, state: &mut NymConnectState) -> NymConnectAction {
    let mut action = NymConnectAction::None;
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);
        ui.label(
            RichText::new("Connect to Nym")
                .font(theme::title_font())
                .color(theme::TEXT),
        );
        ui.add_space(12.0);
        ui.label(
            RichText::new(
                "Traffic is Sphinx-encrypted and routed through the mixnet. Expect higher latency than clearnet. Your Nym address persists across restarts.",
            )
            .color(theme::MUTED),
        );
        ui.add_space(20.0);

        ui.add(
            ProgressBar::new(state.percent as f32 / 100.0)
                .text(format!("{}%", state.percent))
                .desired_width(360.0),
        );
        ui.label(RichText::new(&state.message).color(theme::TEXT));
        if let Some(err) = &state.error {
            ui.label(RichText::new(err).color(theme::DANGER));
        }

        ui.add_space(24.0);
        ui.horizontal(|ui| {
            let can_connect = !state.connecting && !state.connected;
            if ui
                .add_enabled(can_connect, egui::Button::new("Connect").fill(theme::PRIMARY))
                .clicked()
            {
                action = NymConnectAction::Connect;
            }
            if ui
                .add_enabled(
                    state.connected,
                    egui::Button::new("Enter App").fill(theme::SUCCESS),
                )
                .clicked()
            {
                action = NymConnectAction::Continue;
            }
            if theme::danger_button(ui, "Quit").clicked() {
                action = NymConnectAction::Quit;
            }
        });
    });
    action
}
