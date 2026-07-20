use crate::theme;
use egui::{RichText, TextureHandle};
use secure_passage_core::SessionKey;

#[derive(Default)]
pub struct ChatViewState {
    pub mode_host: bool,
    pub session_key: String,
    pub host_address: String,
    pub join_address: String,
    pub join_key: String,
    pub input: String,
    pub messages: Vec<String>,
    pub status: String,
    pub hosting: bool,
    pub connected: bool,
    pub qr_texture: Option<TextureHandle>,
}

pub enum ChatAction {
    None,
    StartHost,
    Stop,
    Join,
    Send(String),
    CopyAddress,
    CopyKey,
    RegenerateKey,
}

pub fn show(ui: &mut egui::Ui, state: &mut ChatViewState, mixnet_connected: bool) -> ChatAction {
    let mut action = ChatAction::None;

    ui.heading(RichText::new("Secure Chat").color(theme::TEXT));
    ui.label(
        RichText::new("End-to-end encrypted messaging over Nym MixnetStream. Share your Nym address and session key out-of-band.")
            .color(theme::MUTED),
    );
    ui.add_space(8.0);

    if !mixnet_connected {
        ui.colored_label(theme::DANGER, "Connect to the Nym mixnet first.");
        return action;
    }

    ui.horizontal(|ui| {
        if theme::mode_tab(ui, state.mode_host, "Host chat").clicked() {
            state.mode_host = true;
        }
        if theme::mode_tab(ui, !state.mode_host, "Join chat").clicked() {
            state.mode_host = false;
        }
    });
    ui.separator();

    if state.mode_host {
        if state.session_key.is_empty() {
            state.session_key = SessionKey::generate().to_base64();
        }
        ui.label(RichText::new("Your Nym address").strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.host_address)
                    .desired_width(420.0)
                    .interactive(false),
            );
            if theme::secondary_button(ui, "Copy").clicked() {
                action = ChatAction::CopyAddress;
            }
        });
        ui.label(RichText::new("Session key").strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.session_key)
                    .desired_width(420.0)
                    .interactive(false),
            );
            if theme::secondary_button(ui, "Copy").clicked() {
                action = ChatAction::CopyKey;
            }
            if theme::secondary_button(ui, "New key").clicked() {
                action = ChatAction::RegenerateKey;
            }
        });

        if let Some(tex) = &state.qr_texture {
            ui.image((tex.id(), egui::vec2(160.0, 160.0)));
        }

        ui.horizontal(|ui| {
            if !state.hosting && !state.connected {
                if theme::primary_button(ui, "Start Hosting").clicked() {
                    action = ChatAction::StartHost;
                }
            } else if theme::danger_button(ui, "Stop").clicked() {
                action = ChatAction::Stop;
            }
        });
    } else {
        ui.label("Host Nym address");
        ui.add(egui::TextEdit::singleline(&mut state.join_address).desired_width(480.0));
        ui.label("Session key");
        ui.add(egui::TextEdit::singleline(&mut state.join_key).desired_width(480.0));
        ui.horizontal(|ui| {
            if !state.connected {
                if theme::primary_button(ui, "Join Chat").clicked() {
                    action = ChatAction::Join;
                }
            } else if theme::danger_button(ui, "Disconnect").clicked() {
                action = ChatAction::Stop;
            }
        });
    }

    ui.label(RichText::new(&state.status).color(theme::ORANGE));
    ui.separator();

    egui::ScrollArea::vertical()
        .max_height(280.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for m in &state.messages {
                ui.label(RichText::new(m).monospace());
            }
        });

    ui.horizontal(|ui| {
        let resp = ui.add(
            egui::TextEdit::singleline(&mut state.input)
                .desired_width(ui.available_width() - 90.0)
                .hint_text("Type a message…"),
        );
        let send = theme::primary_button(ui, "Send").clicked()
            || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
        if send && state.connected && !state.input.trim().is_empty() {
            let content = state.input.trim().to_string();
            state.input.clear();
            action = ChatAction::Send(content);
        }
    });

    action
}

pub fn make_qr_texture(ctx: &egui::Context, payload: &str) -> Option<TextureHandle> {
    let qr = qrcode::QrCode::new(payload.as_bytes()).ok()?;
    let image = qr.render::<image::Luma<u8>>().quiet_zone(true).build();
    let size = [image.width() as usize, image.height() as usize];
    let mut rgba = Vec::with_capacity(size[0] * size[1] * 4);
    for p in image.pixels() {
        let v = p.0[0];
        rgba.extend_from_slice(&[v, v, v, 255]);
    }
    let color = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    Some(ctx.load_texture("chat-qr", color, Default::default()))
}
