use crate::theme;
use egui::{ProgressBar, RichText};

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
                .add_enabled(
                    can_connect,
                    egui::Button::new(
                        RichText::new("Connect")
                            .color(theme::ON_PRIMARY)
                            .strong(),
                    )
                    .fill(theme::primary_fill(ui)),
                )
                .clicked()
            {
                action = NymConnectAction::Connect;
            }
            if theme::enter_app_button(ui, state.connected).clicked() {
                action = NymConnectAction::Continue;
            }
            if theme::danger_button(ui, "Quit").clicked() {
                action = NymConnectAction::Quit;
            }
        });
    });
    action
}
