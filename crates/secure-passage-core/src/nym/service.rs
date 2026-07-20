//! Background task owning MixnetClient and handling UI commands.

use crate::crypto::SessionKey;
use crate::events::{AppEvent, TransferKind};
use crate::protocol::chat::{
    recv_chat_message, recv_handshake, send_chat_message, send_handshake, ChatWireMessage,
};
use crate::protocol::files::{
    file_sha256, hash_password, read_bytes_from_path, recv_file_data, recv_file_meta,
    send_file_data, send_file_meta, write_bytes_to_path, FileMeta, FILE_MAGIC,
};
use crate::protocol::{read_frame, write_frame};
use anyhow::{anyhow, bail, Context, Result};
use nym_sdk::mixnet::{self, MixnetClient, MixnetListener, MixnetStream, Recipient};
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};
use uuid::Uuid;

pub enum NymCommand {
    Connect,
    Disconnect,
    Kill,
    HostChat { session_key: String },
    JoinChat {
        address: String,
        session_key: String,
    },
    SendChat { content: String },
    StopChat,
    HostFile {
        path: PathBuf,
        session_key: String,
        password: Option<String>,
        auto_shutdown: bool,
    },
    ReceiveFile {
        address: String,
        session_key: String,
        dest_dir: PathBuf,
        password: Option<String>,
    },
    StopFile,
}

/// Cloneable handle used by the UI to talk to the background Nym task.
#[derive(Clone)]
pub struct NymHandle {
    cmd_tx: mpsc::UnboundedSender<NymCommand>,
}

impl NymHandle {
    pub fn spawn(event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut worker = NymWorker {
                event_tx,
                client: None,
                listener: None,
                address: None,
                chat_out: None,
                our_id: Uuid::new_v4().to_string(),
                active_cancel: None,
            };
            worker.run(cmd_rx).await;
        });
        Self { cmd_tx }
    }

    pub fn send(&self, cmd: NymCommand) {
        let _ = self.cmd_tx.send(cmd);
    }
}

struct NymWorker {
    event_tx: mpsc::UnboundedSender<AppEvent>,
    client: Option<MixnetClient>,
    listener: Option<MixnetListener>,
    address: Option<String>,
    chat_out: Option<mpsc::UnboundedSender<String>>,
    our_id: String,
    active_cancel: Option<oneshot::Sender<()>>,
}

impl NymWorker {
    fn emit(&self, event: AppEvent) {
        let _ = self.event_tx.send(event);
    }

    fn log(&self, message: impl Into<String>) {
        let message = message.into();
        info!("{message}");
        self.emit(AppEvent::Log {
            level: "INFO".into(),
            message,
        });
    }

    async fn run(&mut self, mut cmd_rx: mpsc::UnboundedReceiver<NymCommand>) {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                NymCommand::Connect => {
                    if let Err(e) = self.connect().await {
                        error!("connect failed: {e:#}");
                        self.emit(AppEvent::ConnectFailed {
                            error: format!("{e:#}"),
                        });
                    }
                }
                NymCommand::Disconnect | NymCommand::Kill => {
                    self.cancel_active();
                    self.disconnect().await;
                }
                NymCommand::HostChat { session_key } => {
                    self.cancel_active();
                    if let Err(e) = self.host_chat(session_key).await {
                        self.emit(AppEvent::ChatError {
                            error: format!("{e:#}"),
                        });
                    }
                }
                NymCommand::JoinChat {
                    address,
                    session_key,
                } => {
                    self.cancel_active();
                    if let Err(e) = self.join_chat(address, session_key).await {
                        self.emit(AppEvent::ChatError {
                            error: format!("{e:#}"),
                        });
                    }
                }
                NymCommand::SendChat { content } => {
                    if let Some(tx) = &self.chat_out {
                        let _ = tx.send(content);
                    } else {
                        self.emit(AppEvent::ChatError {
                            error: "No active chat session".into(),
                        });
                    }
                }
                NymCommand::StopChat => {
                    self.cancel_active();
                    self.chat_out = None;
                    self.emit(AppEvent::ChatPeerLeft);
                }
                NymCommand::HostFile {
                    path,
                    session_key,
                    password,
                    auto_shutdown,
                } => {
                    self.cancel_active();
                    if let Err(e) = self
                        .host_file(path, session_key, password, auto_shutdown)
                        .await
                    {
                        self.emit(AppEvent::TransferError {
                            error: format!("{e:#}"),
                        });
                    }
                }
                NymCommand::ReceiveFile {
                    address,
                    session_key,
                    dest_dir,
                    password,
                } => {
                    self.cancel_active();
                    if let Err(e) = self
                        .receive_file(address, session_key, dest_dir, password)
                        .await
                    {
                        self.emit(AppEvent::TransferError {
                            error: format!("{e:#}"),
                        });
                    }
                }
                NymCommand::StopFile => {
                    self.cancel_active();
                }
            }
        }
        self.disconnect().await;
    }

    fn cancel_active(&mut self) {
        if let Some(tx) = self.active_cancel.take() {
            let _ = tx.send(());
        }
        self.chat_out = None;
    }

    fn storage_dir() -> Result<PathBuf> {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("secure-passage")
            .join("nym");
        std::fs::create_dir_all(&base)?;
        Ok(base)
    }

    async fn connect(&mut self) -> Result<()> {
        if self.client.is_some() {
            self.log("Already connected to the Nym mixnet");
            if let Some(addr) = &self.address {
                self.emit(AppEvent::Connected {
                    address: addr.clone(),
                });
            }
            return Ok(());
        }

        self.emit(AppEvent::ConnectProgress {
            percent: 10,
            message: "Preparing persistent identity…".into(),
        });

        let config_dir = Self::storage_dir()?;
        let storage_paths = mixnet::StoragePaths::new_from_dir(&config_dir)
            .map_err(|e| anyhow!("storage paths: {e}"))?;

        self.emit(AppEvent::ConnectProgress {
            percent: 30,
            message: "Building mixnet client…".into(),
        });

        let disconnected = mixnet::MixnetClientBuilder::new_with_default_storage(storage_paths)
            .await
            .map_err(|e| anyhow!("storage builder: {e}"))?
            .build()
            .map_err(|e| anyhow!("build client: {e}"))?;

        self.emit(AppEvent::ConnectProgress {
            percent: 60,
            message: "Connecting to gateway (this may take a minute)…".into(),
        });

        let client = disconnected
            .connect_to_mixnet()
            .await
            .map_err(|e| anyhow!("connect_to_mixnet: {e}"))?;

        let address = client.nym_address().to_string();
        self.log(format!("Connected. Nym address: {address}"));

        self.emit(AppEvent::ConnectProgress {
            percent: 100,
            message: "Connected".into(),
        });
        self.emit(AppEvent::Connected {
            address: address.clone(),
        });

        self.address = Some(address);
        self.client = Some(client);
        self.listener = None;
        Ok(())
    }

    async fn disconnect(&mut self) {
        self.cancel_active();
        self.listener = None;
        if let Some(client) = self.client.take() {
            self.log("Disconnecting from Nym mixnet…");
            client.disconnect().await;
        }
        self.address = None;
        self.emit(AppEvent::Disconnected);
    }

    fn ensure_listener(&mut self) -> Result<()> {
        if self.listener.is_some() {
            return Ok(());
        }
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected to the Nym mixnet"))?;
        let listener = client
            .listener()
            .map_err(|e| anyhow!("listener: {e}"))?;
        self.listener = Some(listener);
        Ok(())
    }

    async fn accept_stream(&mut self) -> Result<MixnetStream> {
        self.ensure_listener()?;
        let listener = self
            .listener
            .as_mut()
            .ok_or_else(|| anyhow!("no listener"))?;

        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        self.active_cancel = Some(cancel_tx);

        tokio::select! {
            biased;
            _ = cancel_rx => {
                bail!("cancelled while waiting for peer");
            }
            s = listener.accept() => {
                s.ok_or_else(|| anyhow!("listener closed"))
            }
        }
    }

    async fn open_outbound(&mut self, address: &str) -> Result<MixnetStream> {
        let recipient: Recipient = address
            .parse()
            .map_err(|e| anyhow!("invalid Nym address: {e}"))?;
        // Stream mode must be active on this client too
        self.ensure_listener()?;
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;
        client
            .open_stream(recipient, None)
            .await
            .map_err(|e| anyhow!("open_stream: {e}"))
    }

    async fn host_chat(&mut self, session_key_b64: String) -> Result<()> {
        let key = SessionKey::from_base64(&session_key_b64)?;
        let address = self
            .address
            .clone()
            .ok_or_else(|| anyhow!("Not connected"))?;

        self.emit(AppEvent::ChatHosting {
            address: address.clone(),
            session_key: session_key_b64,
        });
        self.log("Chat host waiting for peer stream…");

        let stream = self.accept_stream().await?;
        self.log("Peer connected to chat");
        self.spawn_chat_session(stream, key).await
    }

    async fn join_chat(&mut self, address: String, session_key_b64: String) -> Result<()> {
        let key = SessionKey::from_base64(&session_key_b64)?;
        self.log(format!("Opening chat stream to {address}…"));
        let stream = self.open_outbound(&address).await?;
        self.emit(AppEvent::ChatJoined {
            peer: address,
        });
        self.spawn_chat_session(stream, key).await
    }

    async fn spawn_chat_session(&mut self, mut stream: MixnetStream, key: SessionKey) -> Result<()> {
        send_handshake(&mut stream, &key, &self.our_id, "peer").await?;
        let peer_hs = recv_handshake(&mut stream, &key).await?;
        self.log(format!("Chat handshake ok with {}", peer_hs.sender_id));
        self.emit(AppEvent::ChatJoined {
            peer: peer_hs.sender_id.clone(),
        });

        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
        self.chat_out = Some(out_tx);

        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        self.active_cancel = Some(cancel_tx);

        let event_tx = self.event_tx.clone();
        let our_id = self.our_id.clone();
        let key_recv = key.clone();
        let key_send = key;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = &mut cancel_rx => {
                        let _ = event_tx.send(AppEvent::ChatPeerLeft);
                        break;
                    }
                    maybe_msg = out_rx.recv() => {
                        match maybe_msg {
                            Some(content) => {
                                let msg = ChatWireMessage::new(&our_id, content);
                                if let Err(e) = send_chat_message(&mut stream, &key_send, &msg).await {
                                    let _ = event_tx.send(AppEvent::ChatError { error: format!("{e:#}") });
                                    break;
                                }
                                let _ = event_tx.send(AppEvent::ChatMessage {
                                    sender_id: msg.sender_id,
                                    content: msg.content,
                                    timestamp: msg.timestamp,
                                    message_id: msg.message_id,
                                });
                            }
                            None => break,
                        }
                    }
                    incoming = recv_chat_message(&mut stream, &key_recv) => {
                        match incoming {
                            Ok(msg) => {
                                let _ = event_tx.send(AppEvent::ChatMessage {
                                    sender_id: msg.sender_id,
                                    content: msg.content,
                                    timestamp: msg.timestamp,
                                    message_id: msg.message_id,
                                });
                            }
                            Err(e) => {
                                let _ = event_tx.send(AppEvent::ChatError {
                                    error: format!("chat stream ended: {e:#}"),
                                });
                                let _ = event_tx.send(AppEvent::ChatPeerLeft);
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn host_file(
        &mut self,
        path: PathBuf,
        session_key_b64: String,
        password: Option<String>,
        auto_shutdown: bool,
    ) -> Result<()> {
        let key = SessionKey::from_base64(&session_key_b64)?;
        let address = self
            .address
            .clone()
            .ok_or_else(|| anyhow!("Not connected"))?;
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();

        self.emit(AppEvent::FileHosting {
            address: address.clone(),
            session_key: session_key_b64,
            file_name: file_name.clone(),
        });

        let data = read_bytes_from_path(&path).await.context("read file")?;
        let meta = FileMeta {
            magic: FILE_MAGIC.to_string(),
            file_name: file_name.clone(),
            size: data.len() as u64,
            sha256: file_sha256(&data),
            password_hash: password.as_deref().map(hash_password),
        };

        self.log(format!(
            "File host ready: {} ({} bytes)",
            file_name,
            data.len()
        ));

        let mut stream = self.accept_stream().await?;
        self.log("Peer connected for file transfer");

        if let Some(expected) = &meta.password_hash {
            let challenge = read_frame(&mut stream).await?;
            let got = String::from_utf8_lossy(&challenge).to_string();
            if &got != expected {
                self.emit(AppEvent::TransferError {
                    error: "Password mismatch".into(),
                });
                return Ok(());
            }
        }

        send_file_meta(&mut stream, &key, &meta).await?;
        let event_tx = self.event_tx.clone();
        let fname = file_name.clone();
        send_file_data(&mut stream, &key, &data, |done, total| {
            let _ = event_tx.send(AppEvent::TransferProgress {
                kind: TransferKind::Send,
                file_name: fname.clone(),
                bytes_done: done,
                bytes_total: total,
            });
        })
        .await?;

        self.emit(AppEvent::TransferComplete {
            kind: TransferKind::Send,
            file_name,
            path: path.display().to_string(),
        });
        self.log("File send complete");

        if auto_shutdown {
            self.log("Auto-shutdown after transfer");
            self.cancel_active();
        }
        Ok(())
    }

    async fn receive_file(
        &mut self,
        address: String,
        session_key_b64: String,
        dest_dir: PathBuf,
        password: Option<String>,
    ) -> Result<()> {
        let key = SessionKey::from_base64(&session_key_b64)?;
        let mut stream = self.open_outbound(&address).await?;

        if let Some(ref pw) = password {
            let hash = hash_password(pw);
            write_frame(&mut stream, hash.as_bytes()).await?;
        }

        let meta = recv_file_meta(&mut stream, &key).await?;
        if meta.password_hash.is_some() && password.is_none() {
            bail!("This file requires a password");
        }

        let event_tx = self.event_tx.clone();
        let fname = meta.file_name.clone();
        let data = recv_file_data(&mut stream, &key, meta.size, |done, total| {
            let _ = event_tx.send(AppEvent::TransferProgress {
                kind: TransferKind::Receive,
                file_name: fname.clone(),
                bytes_done: done,
                bytes_total: total,
            });
        })
        .await?;

        let digest = file_sha256(&data);
        if digest != meta.sha256 {
            bail!("checksum mismatch");
        }

        let dest = dest_dir.join(&meta.file_name);
        write_bytes_to_path(&dest, &data).await?;
        self.emit(AppEvent::TransferComplete {
            kind: TransferKind::Receive,
            file_name: meta.file_name,
            path: dest.display().to_string(),
        });
        self.log(format!("File saved to {}", dest.display()));
        Ok(())
    }
}
