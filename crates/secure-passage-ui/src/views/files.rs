use crate::theme;
use crate::views::chat::make_qr_texture;
use egui::{ProgressBar, RichText, TextureHandle};
use secure_passage_core::SessionKey;
use std::path::PathBuf;

#[derive(Default)]
pub struct FilesViewState {
    pub mode_host: bool,
    pub selected_path: Option<PathBuf>,
    pub session_key: String,
    pub host_address: String,
    pub password: String,
    pub auto_shutdown: bool,
    pub join_address: String,
    pub join_key: String,
    pub join_password: String,
    pub dest_dir: Option<PathBuf>,
    pub status: String,
    pub hosting: bool,
    pub transferring: bool,
    pub progress: f32,
    pub progress_label: String,
    pub file_name: String,
    pub qr_texture: Option<TextureHandle>,
}

pub enum FilesAction {
    None,
    PickFile,
    PickDest,
    StartHost,
    StopHost,
    StartReceive,
    CopyAddress,
    CopyKey,
    RegenerateKey,
}

pub fn show(ui: &mut egui::Ui, state: &mut FilesViewState, mixnet_connected: bool) -> FilesAction {
    let mut action = FilesAction::None;

    ui.heading(RichText::new("File Sharing").color(theme::TEXT));
    ui.label(
        RichText::new(
            "Share files with AES-256-GCM encryption over the Nym mixnet. No local HTTP server — pure MixnetStream transfer.",
        )
        .color(theme::MUTED),
    );
    ui.add_space(8.0);

    if !mixnet_connected {
        ui.colored_label(theme::DANGER, "Connect to the Nym mixnet first.");
        return action;
    }

    ui.horizontal(|ui| {
        if ui.selectable_label(state.mode_host, "Send / Host").clicked() {
            state.mode_host = true;
        }
        if ui
            .selectable_label(!state.mode_host, "Receive")
            .clicked()
        {
            state.mode_host = false;
        }
    });
    ui.separator();

    if state.mode_host {
        if state.session_key.is_empty() {
            state.session_key = SessionKey::generate().to_base64();
        }

        ui.horizontal(|ui| {
            if theme::secondary_button(ui, "Choose file…").clicked() {
                action = FilesAction::PickFile;
            }
            let name = state
                .selected_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "No file selected".into());
            ui.label(name);
        });

        ui.label(RichText::new("Your Nym address").strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.host_address)
                    .desired_width(420.0)
                    .interactive(false),
            );
            if theme::secondary_button(ui, "Copy").clicked() {
                action = FilesAction::CopyAddress;
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
                action = FilesAction::CopyKey;
            }
            if theme::secondary_button(ui, "New key").clicked() {
                action = FilesAction::RegenerateKey;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Optional password:");
            ui.add(
                egui::TextEdit::singleline(&mut state.password)
                    .password(true)
                    .desired_width(200.0),
            );
            ui.checkbox(&mut state.auto_shutdown, "Auto-shutdown after transfer");
        });

        if let Some(tex) = &state.qr_texture {
            ui.image((tex.id(), egui::vec2(160.0, 160.0)));
        }

        ui.horizontal(|ui| {
            if !state.hosting && !state.transferring {
                let can = state.selected_path.is_some();
                if ui
                    .add_enabled(can, egui::Button::new("Start Sharing").fill(theme::PRIMARY))
                    .clicked()
                {
                    action = FilesAction::StartHost;
                }
            } else if theme::danger_button(ui, "Stop").clicked() {
                action = FilesAction::StopHost;
            }
        });
    } else {
        ui.label("Host Nym address");
        ui.add(egui::TextEdit::singleline(&mut state.join_address).desired_width(480.0));
        ui.label("Session key");
        ui.add(egui::TextEdit::singleline(&mut state.join_key).desired_width(480.0));
        ui.horizontal(|ui| {
            ui.label("Password (if required):");
            ui.add(
                egui::TextEdit::singleline(&mut state.join_password)
                    .password(true)
                    .desired_width(200.0),
            );
        });
        ui.horizontal(|ui| {
            if theme::secondary_button(ui, "Choose download folder…").clicked() {
                action = FilesAction::PickDest;
            }
            let dest = state
                .dest_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Not set".into());
            ui.label(dest);
        });
        if !state.transferring {
            let can = state.dest_dir.is_some()
                && !state.join_address.is_empty()
                && !state.join_key.is_empty();
            if ui
                .add_enabled(can, egui::Button::new("Receive File").fill(theme::PRIMARY))
                .clicked()
            {
                action = FilesAction::StartReceive;
            }
        }
    }

    if state.transferring || state.progress > 0.0 {
        ui.add(
            ProgressBar::new(state.progress)
                .text(&state.progress_label)
                .desired_width(400.0),
        );
    }
    ui.label(RichText::new(&state.status).color(theme::ORANGE));

    action
}

pub fn refresh_file_qr(ctx: &egui::Context, state: &mut FilesViewState) {
    if state.host_address.is_empty() || state.session_key.is_empty() {
        return;
    }
    let payload = format!("{}|{}", state.host_address, state.session_key);
    state.qr_texture = make_qr_texture(ctx, &payload);
}
