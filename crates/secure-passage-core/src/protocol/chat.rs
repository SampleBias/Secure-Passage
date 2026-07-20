//! Encrypted chat frames over MixnetStream.

use crate::crypto::SessionKey;
use crate::protocol::{read_frame, write_frame};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use uuid::Uuid;

pub const HANDSHAKE_MAGIC: &str = "SPCHAT01";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHandshake {
    pub magic: String,
    pub sender_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatWireMessage {
    pub sender_id: String,
    pub content: String,
    pub timestamp: f64,
    pub message_id: String,
}

impl ChatWireMessage {
    pub fn new(sender_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            sender_id: sender_id.into(),
            content: content.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            message_id: Uuid::new_v4().to_string(),
        }
    }
}

pub async fn send_handshake<W: AsyncWrite + Unpin>(
    writer: &mut W,
    key: &SessionKey,
    sender_id: &str,
    role: &str,
) -> Result<()> {
    let hs = ChatHandshake {
        magic: HANDSHAKE_MAGIC.to_string(),
        sender_id: sender_id.to_string(),
        role: role.to_string(),
    };
    let plain = serde_json::to_vec(&hs)?;
    let enc = key.encrypt(&plain).context("encrypt handshake")?;
    write_frame(writer, &enc).await?;
    Ok(())
}

pub async fn recv_handshake<R: AsyncRead + Unpin>(
    reader: &mut R,
    key: &SessionKey,
) -> Result<ChatHandshake> {
    let enc = read_frame(reader).await?;
    let plain = key.decrypt(&enc).context("decrypt handshake")?;
    let hs: ChatHandshake = serde_json::from_slice(&plain)?;
    if hs.magic != HANDSHAKE_MAGIC {
        bail!("invalid chat handshake magic");
    }
    Ok(hs)
}

pub async fn send_chat_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    key: &SessionKey,
    msg: &ChatWireMessage,
) -> Result<()> {
    let plain = serde_json::to_vec(msg)?;
    let enc = key.encrypt(&plain).context("encrypt chat")?;
    write_frame(writer, &enc).await?;
    Ok(())
}

pub async fn recv_chat_message<R: AsyncRead + Unpin>(
    reader: &mut R,
    key: &SessionKey,
) -> Result<ChatWireMessage> {
    let enc = read_frame(reader).await?;
    let plain = key.decrypt(&enc).context("decrypt chat")?;
    Ok(serde_json::from_slice(&plain)?)
}
