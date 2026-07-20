use crate::theme;
use crate::views::chat::{self, ChatAction, ChatViewState};
use crate::views::dialogs::{self, NymConnectAction, NymConnectState};
use crate::views::files::{self, FilesAction, FilesViewState};
use crate::views::logs::LogsState;
use crate::views::nav::{self, Page};
use crate::views::status::{self, StatusAction, StatusBarState};
use crate::views::{banner, logs};
use secure_passage_core::{AppEvent, NymCommand, NymHandle, SessionKey, TransferKind};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    NymConnect,
    Main,
}

pub struct SecurePassageApp {
    screen: Screen,
    /// Kept alive so the multi-thread Tokio runtime outlives Nym background tasks.
    _runtime: tokio::runtime::Runtime,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    nym: NymHandle,
    nym_connect: NymConnectState,
    page: Page,
    status: StatusBarState,
    logs: LogsState,
    chat: ChatViewState,
    files: FilesViewState,
    mixnet_address: Option<String>,
    mixnet_connected: bool,
}

pub fn create(cc: &eframe::CreationContext<'_>) -> SecurePassageApp {
    theme::apply(&cc.egui_ctx);
    egui_extras::install_image_loaders(&cc.egui_ctx);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let nym = {
        let _guard = runtime.enter();
        NymHandle::spawn(event_tx.clone())
    };

    SecurePassageApp {
        screen: Screen::NymConnect,
        _runtime: runtime,
        event_rx,
        nym,
        nym_connect: NymConnectState::default(),
        page: Page::Files,
        status: StatusBarState::default(),
        logs: LogsState::default(),
        chat: ChatViewState::default(),
        files: FilesViewState {
            auto_shutdown: true,
            ..Default::default()
        },
        mixnet_address: None,
        mixnet_connected: false,
    }
}

impl SecurePassageApp {
    fn drain_events(&mut self, ctx: &egui::Context) {
        while let Ok(ev) = self.event_rx.try_recv() {
            self.handle_event(ctx, ev);
        }
    }

    fn handle_event(&mut self, ctx: &egui::Context, ev: AppEvent) {
        match ev {
            AppEvent::Log { level, message } => self.logs.push(&level, &message),
            AppEvent::ConnectProgress { percent, message } => {
                self.nym_connect.connecting = percent < 100;
                self.nym_connect.percent = percent;
                self.nym_connect.message = message;
            }
            AppEvent::Connected { address } => {
                self.nym_connect.connecting = false;
                self.nym_connect.connected = true;
                self.nym_connect.percent = 100;
                self.nym_connect.message = format!("Connected: {address}");
                self.nym_connect.error = None;
                self.mixnet_connected = true;
                self.mixnet_address = Some(address.clone());
                self.status.mixnet = format!("Connected to Nym — {address}");
                self.chat.host_address = address.clone();
                self.files.host_address = address;
                self.refresh_qrs(ctx);
                self.logs.push("INFO", "Nym mixnet connected");
            }
            AppEvent::Disconnected => {
                self.mixnet_connected = false;
                self.mixnet_address = None;
                self.nym_connect = NymConnectState::default();
                self.status.mixnet = "Nym not connected".into();
                self.chat.hosting = false;
                self.chat.connected = false;
                self.files.hosting = false;
                self.files.transferring = false;
                self.logs.push("INFO", "Disconnected from Nym");
            }
            AppEvent::ConnectFailed { error } => {
                self.nym_connect.connecting = false;
                self.nym_connect.error = Some(error.clone());
                self.logs.push("ERROR", &error);
            }
            AppEvent::ChatHosting {
                address,
                session_key,
            } => {
                self.chat.hosting = true;
                self.chat.host_address = address;
                self.chat.session_key = session_key;
                self.chat.status = "Hosting — waiting for peer…".into();
                self.refresh_qrs(ctx);
            }
            AppEvent::ChatJoined { peer } => {
                self.chat.connected = true;
                self.chat.hosting = false;
                self.chat.status = format!("Connected with {peer}");
            }
            AppEvent::ChatMessage {
                sender_id,
                content,
                ..
            } => {
                let short = if sender_id.len() > 8 {
                    &sender_id[..8]
                } else {
                    &sender_id
                };
                self.chat.messages.push(format!("[{short}] {content}"));
            }
            AppEvent::ChatPeerLeft => {
                self.chat.connected = false;
                self.chat.hosting = false;
                self.chat.status = "Chat ended".into();
            }
            AppEvent::ChatError { error } => {
                self.chat.status = error.clone();
                self.logs.push("ERROR", &error);
            }
            AppEvent::FileHosting {
                address,
                session_key,
                file_name,
            } => {
                self.files.hosting = true;
                self.files.host_address = address;
                self.files.session_key = session_key;
                self.files.file_name = file_name.clone();
                self.files.status = format!("Hosting {file_name} — waiting for peer…");
                self.refresh_qrs(ctx);
            }
            AppEvent::TransferProgress {
                kind,
                file_name,
                bytes_done,
                bytes_total,
            } => {
                self.files.transferring = true;
                self.files.progress = if bytes_total > 0 {
                    bytes_done as f32 / bytes_total as f32
                } else {
                    0.0
                };
                let verb = match kind {
                    TransferKind::Send => "Sending",
                    TransferKind::Receive => "Receiving",
                };
                self.files.progress_label =
                    format!("{verb} {file_name}: {bytes_done}/{bytes_total}");
            }
            AppEvent::TransferComplete {
                kind,
                file_name,
                path,
            } => {
                self.files.transferring = false;
                self.files.hosting = false;
                self.files.progress = 1.0;
                let verb = match kind {
                    TransferKind::Send => "Sent",
                    TransferKind::Receive => "Received",
                };
                self.files.status = format!("{verb} {file_name} → {path}");
                self.logs.push("INFO", &self.files.status.clone());
            }
            AppEvent::TransferError { error } => {
                self.files.transferring = false;
                self.files.hosting = false;
                self.files.status = error.clone();
                self.logs.push("ERROR", &error);
            }
        }
    }

    fn refresh_qrs(&mut self, ctx: &egui::Context) {
        if !self.chat.host_address.is_empty() && !self.chat.session_key.is_empty() {
            let payload = format!("{}|{}", self.chat.host_address, self.chat.session_key);
            self.chat.qr_texture = chat::make_qr_texture(ctx, &payload);
        }
        files::refresh_file_qr(ctx, &mut self.files);
    }

    fn copy_text(text: &str) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text.to_string());
        }
    }
}

impl eframe::App for SecurePassageApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events(ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        match self.screen {
            Screen::NymConnect => {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(theme::BG))
                    .show(ui, |ui| {
                        match dialogs::show_nym_connect(ui, &mut self.nym_connect) {
                            NymConnectAction::Connect => {
                                self.nym_connect.connecting = true;
                                self.nym_connect.error = None;
                                self.nym_connect.percent = 5;
                                self.nym_connect.message = "Starting…".into();
                                self.nym.send(NymCommand::Connect);
                            }
                            NymConnectAction::Continue => self.screen = Screen::Main,
                            NymConnectAction::Quit => {
                                self.nym.send(NymCommand::Kill);
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            NymConnectAction::None => {}
                        }
                    });
            }
            Screen::Main => {
                egui::Panel::bottom("status")
                    .exact_size(40.0)
                    .frame(egui::Frame::NONE.fill(theme::PANEL).inner_margin(8.0))
                    .show(ui, |ui| match status::show(ui, &self.status) {
                        StatusAction::Logs => self.logs.open = true,
                        StatusAction::Kill => {
                            self.nym.send(NymCommand::Kill);
                            self.screen = Screen::NymConnect;
                        }
                        StatusAction::None => {}
                    });

                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(theme::BG).inner_margin(20.0))
                    .show(ui, |ui| {
                        banner::show(ui);
                        nav::show(ui, &mut self.page);
                        ui.add_space(12.0);

                        egui::ScrollArea::vertical().show(ui, |ui| match self.page {
                            Page::Files => handle_files(self, ui, &ctx),
                            Page::Chat => handle_chat(self, ui, &ctx),
                            Page::Hosting => {
                                ui.heading("Website Hosting (in dev)");
                                ui.label(
                                    egui::RichText::new("Coming soon — deferred past MVP.")
                                        .color(theme::MUTED),
                                );
                            }
                            Page::Browser => {
                                ui.heading("Nym Browser (in dev)");
                                ui.label(
                                    egui::RichText::new(
                                        "Coming soon — SOCKS / IPR browsing in a later phase.",
                                    )
                                    .color(theme::MUTED),
                                );
                            }
                        });
                    });

                logs::show(&ctx, &mut self.logs);
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.nym.send(NymCommand::Kill);
    }
}

fn handle_chat(app: &mut SecurePassageApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    if app.chat.host_address.is_empty() {
        if let Some(a) = &app.mixnet_address {
            app.chat.host_address = a.clone();
        }
    }
    match chat::show(ui, &mut app.chat, app.mixnet_connected) {
        ChatAction::StartHost => {
            if app.chat.session_key.is_empty() {
                app.chat.session_key = SessionKey::generate().to_base64();
            }
            app.nym.send(NymCommand::HostChat {
                session_key: app.chat.session_key.clone(),
            });
            app.refresh_qrs(ctx);
        }
        ChatAction::Join => {
            app.nym.send(NymCommand::JoinChat {
                address: app.chat.join_address.trim().to_string(),
                session_key: app.chat.join_key.trim().to_string(),
            });
        }
        ChatAction::Send(content) => app.nym.send(NymCommand::SendChat { content }),
        ChatAction::Stop => {
            app.nym.send(NymCommand::StopChat);
            app.chat.hosting = false;
            app.chat.connected = false;
        }
        ChatAction::CopyAddress => SecurePassageApp::copy_text(&app.chat.host_address),
        ChatAction::CopyKey => SecurePassageApp::copy_text(&app.chat.session_key),
        ChatAction::RegenerateKey => {
            app.chat.session_key = SessionKey::generate().to_base64();
            app.refresh_qrs(ctx);
        }
        ChatAction::None => {}
    }
}

fn handle_files(app: &mut SecurePassageApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    if app.files.host_address.is_empty() {
        if let Some(a) = &app.mixnet_address {
            app.files.host_address = a.clone();
        }
    }
    match files::show(ui, &mut app.files, app.mixnet_connected) {
        FilesAction::PickFile => {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                app.files.selected_path = Some(path);
            }
        }
        FilesAction::PickDest => {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.files.dest_dir = Some(path);
            }
        }
        FilesAction::StartHost => {
            if let Some(path) = app.files.selected_path.clone() {
                if app.files.session_key.is_empty() {
                    app.files.session_key = SessionKey::generate().to_base64();
                }
                let password = if app.files.password.is_empty() {
                    None
                } else {
                    Some(app.files.password.clone())
                };
                app.nym.send(NymCommand::HostFile {
                    path,
                    session_key: app.files.session_key.clone(),
                    password,
                    auto_shutdown: app.files.auto_shutdown,
                });
                app.refresh_qrs(ctx);
            }
        }
        FilesAction::StopHost => {
            app.nym.send(NymCommand::StopFile);
            app.files.hosting = false;
            app.files.transferring = false;
        }
        FilesAction::StartReceive => {
            if let Some(dest_dir) = app.files.dest_dir.clone() {
                let password = if app.files.join_password.is_empty() {
                    None
                } else {
                    Some(app.files.join_password.clone())
                };
                app.nym.send(NymCommand::ReceiveFile {
                    address: app.files.join_address.trim().to_string(),
                    session_key: app.files.join_key.trim().to_string(),
                    dest_dir,
                    password,
                });
                app.files.transferring = true;
                app.files.status = "Connecting to host…".into();
            }
        }
        FilesAction::CopyAddress => SecurePassageApp::copy_text(&app.files.host_address),
        FilesAction::CopyKey => SecurePassageApp::copy_text(&app.files.session_key),
        FilesAction::RegenerateKey => {
            app.files.session_key = SessionKey::generate().to_base64();
            app.refresh_qrs(ctx);
        }
        FilesAction::None => {}
    }
}
